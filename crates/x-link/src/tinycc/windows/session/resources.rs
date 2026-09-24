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
