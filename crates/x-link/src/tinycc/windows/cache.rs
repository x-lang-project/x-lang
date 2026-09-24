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

mod object_format;
pub(crate) use object_format::*;
