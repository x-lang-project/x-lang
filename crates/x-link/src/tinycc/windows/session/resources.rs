use std::fs::{File, OpenOptions};
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;

pub(super) struct Resources {
    pub(super) root: PathBuf,
    base: PathBuf,
    guards: Vec<File>,
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
