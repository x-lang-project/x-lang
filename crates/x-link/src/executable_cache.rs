use std::io;

use std::{
    cell::RefCell,
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::NativeToolchain;

thread_local! {
    static CAPTURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static RUNTIME: RefCell<Option<(PathBuf, String)>> = const { RefCell::new(None) };
    static FILES: RefCell<Option<BTreeMap<PathBuf, Option<Vec<u8>>>>> = const { RefCell::new(None) };
}
#[must_use]
pub struct Capture;
impl Drop for Capture {
    fn drop(&mut self) {
        CAPTURE.with(|slot| slot.set(false));
        RUNTIME.with(|slot| *slot.borrow_mut() = None);
        FILES.with(|slot| *slot.borrow_mut() = None);
    }
}

pub fn begin_capture() -> Capture {
    CAPTURE.with(|slot| slot.set(true));
    RUNTIME.with(|slot| *slot.borrow_mut() = None);
    FILES.with(|slot| *slot.borrow_mut() = None);

    Capture
}

pub(crate) fn capturing() -> bool {
    CAPTURE.with(|slot| slot.get())
}

pub(crate) fn valid_files(files: &BTreeMap<PathBuf, Option<Vec<u8>>>) -> bool {
    files.iter().all(|(path, identity)| match identity {
        Some(expected) => NativeToolchain::file_identity(path).as_ref() == Some(expected),
        // Missing journal metadata does not prove an optional search path is absent.
        None => {
            fs::symlink_metadata(path).is_err_and(|error| error.kind() == io::ErrorKind::NotFound)
        }
    })
}

pub(crate) fn record_runtime(object: &Path) {
    let record = object.parent().and_then(|entry| {
        Some((
            entry.to_path_buf(),
            entry.parent()?.file_name()?.to_str()?.to_owned(),
        ))
    });
    RUNTIME.with(|slot| *slot.borrow_mut() = record);
}

pub(crate) fn record_files(files: Option<BTreeMap<PathBuf, Option<Vec<u8>>>>) {
    FILES.with(|slot| *slot.borrow_mut() = files);
}

pub(crate) fn library_files(
    arguments: &[String],
    paths: impl Iterator<Item = PathBuf>,
) -> Option<BTreeMap<PathBuf, Option<Vec<u8>>>> {
    let mut files = BTreeMap::new();

    for path in paths {
        files.insert(path.clone(), Some(NativeToolchain::file_identity(&path)?));
    }
    // Track directory additions and library replacements to preserve search precedence.
    for directory in arguments
        .iter()
        .filter_map(|argument| argument.strip_prefix("-libpath:"))
    {
        let directory = PathBuf::from(directory);

        if !directory.exists() {
            files.insert(directory.clone(), None);
            let parent = directory.ancestors().find(|path| path.is_dir())?;
            files.insert(
                parent.to_path_buf(),
                Some(NativeToolchain::file_identity(parent)?),
            );
            continue;
        }

        files.insert(
            directory.clone(),
            Some(NativeToolchain::file_identity(&directory)?),
        );

        for entry in fs::read_dir(directory).ok()? {
            let path = entry.ok()?.path();

            if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("lib"))
            {
                files.insert(path.clone(), Some(NativeToolchain::file_identity(&path)?));
            }
        }
    }

    Some(files)
}

pub fn toolchain_identity() -> Option<Vec<u8>> {
    if !cfg!(all(windows, target_arch = "x86_64")) || env::var_os("X_CLANG").is_some() {
        return None;
    }

    let toolchain = NativeToolchain::discover().ok()?;
    let compiler = toolchain.compiler().canonicalize().ok()?;
    let directory = compiler.parent()?;
    let mut identity = NativeToolchain::discovery_fingerprint()?;
    let executable = env::current_exe().ok()?;
    let loaded_llvm = executable.parent()?.join("LLVM-C.dll");

    for path in [
        compiler.clone(),
        directory.join("lld-link.exe"),
        directory.join("LLVM-C.dll"),
        loaded_llvm,
        executable,
    ] {
        identity.extend(NativeToolchain::file_identity(&path)?);
    }

    identity.extend(toolchain.cache_identity()?);

    Some(identity)
}
