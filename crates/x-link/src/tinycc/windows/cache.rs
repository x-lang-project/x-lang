use std::os::windows::fs::MetadataExt;
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use super::*;

mod identity;

pub(super) struct RuntimeObject {
    pub(super) path: PathBuf,
    temporary: bool,
    _package_guards: Option<Vec<fs::File>>,
}

#[cfg(feature = "tinycc-session")]
impl RuntimeObject {
    pub(super) fn is_guarded(&self) -> bool {
        self._package_guards.is_some()
    }
}

impl Drop for RuntimeObject {
    fn drop(&mut self) {
        if self.temporary {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn verified_object(path: &Path, digest_path: &Path) -> bool {
    let Ok(bytes) = fs::read(path) else {
        return false;
    };

    elf_object(&bytes)
        && fs::read_to_string(digest_path)
            .is_ok_and(|expected| expected == hex(&Sha256::digest(bytes)))
}

mod object_format;
pub(crate) use object_format::*;
