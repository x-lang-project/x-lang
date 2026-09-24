use std::fs::{File, OpenOptions};
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;

static NEXT_SESSION: AtomicU64 = AtomicU64::new(0);

fn create_directory(base: &Path, sequence: &AtomicU64) -> Result<PathBuf, CompilationFailure> {
    // Reserve a fresh directory because stopped workers can leave snapshots behind.
    for _ in 0..64 {
        let root = base.join(format!(
            "session-{}-{}",
            std::process::id(),
            sequence.fetch_add(1, Ordering::Relaxed)
        ));

        match fs::create_dir(&root) {
            Ok(()) => return Ok(root),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(failure(error)),
        }
    }

    Err(failure("cannot reserve a private TinyCC session directory"))
}

pub(super) struct Resources {
    pub(super) root: PathBuf,
    base: PathBuf,
    guards: Vec<File>,
}

pub(super) fn read_guard(path: &Path) -> Result<File, CompilationFailure> {
    OpenOptions::new()
        .access_mode(1)
        .share_mode(1)
        .open(path)
        .map_err(failure)
}

impl Resources {
    pub(super) fn snapshot(source: &Path) -> Result<Self, CompilationFailure> {
        let base = env::var_os("X_RUNTIME_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir)
            .join("x-tinycc-private-sessions");
        fs::create_dir_all(&base).map_err(failure)?;
        let base = base.canonicalize().map_err(failure)?;
        let root = create_directory(&base, &NEXT_SESSION)?;
        let mut resources = Self {
            root,
            base,
            guards: Vec::new(),
        };
        let mut pending = ["include", "lib", "libtcc"]
            .map(|name| source.join(name))
            .to_vec();

        while let Some(path) = pending.pop() {
            let metadata = path.symlink_metadata().map_err(failure)?;

            if metadata.file_attributes() & 0x400 != 0 {
                return Err(failure("session package snapshot contains a reparse point"));
            }

            let destination = resources
                .root
                .join(path.strip_prefix(source).map_err(failure)?);

            if metadata.is_dir() {
                fs::create_dir(&destination).map_err(failure)?;

                for entry in fs::read_dir(path).map_err(failure)? {
                    pending.push(entry.map_err(failure)?.path());
                }
            } else if metadata.is_file() {
                fs::copy(&path, &destination).map_err(failure)?;
                resources.guards.push(read_guard(&destination)?);
            } else {
                return Err(failure("unsupported file type in session package snapshot"));
            }
        }

        Ok(resources)
    }
}

impl Drop for Resources {
    fn drop(&mut self) {
        self.guards.clear();
        // Remove only the owned child; never traverse a substituted reparse point.
        let owned = self.root.parent() == Some(self.base.as_path())
            && self
                .root
                .symlink_metadata()
                .is_ok_and(|metadata| metadata.is_dir() && metadata.file_attributes() & 0x400 == 0);

        if owned {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
