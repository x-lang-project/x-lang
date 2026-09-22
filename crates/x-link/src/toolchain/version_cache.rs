//! Explicit X_CLANG wrappers bypass this cache because their versions may depend on other inputs.
use std::env;
use std::fs;
#[cfg(windows)]
use std::fs::File;
use std::io;
#[cfg(windows)]
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[cfg(windows)]
use crate::windows_file::*;

use storage::*;
use x_utils::tools::clang_is_pinned;

static NEXT_ENTRY: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Identity {
    path: PathBuf,
    size: u64,
    volume: u32,
    file_identifier: u64,
    modified: i64,
    changed: i64,
    usn: i64,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    identity: Identity,
    version: String,
}

mod storage;
