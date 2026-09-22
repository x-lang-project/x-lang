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
#[cfg(windows)]
use identity::*;
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

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn DeviceIoControl(
        handle: *mut std::ffi::c_void,
        code: u32,
        input: *const std::ffi::c_void,
        input_size: u32,
        output: *mut std::ffi::c_void,
        output_size: u32,
        returned: *mut u32,
        overlapped: *mut std::ffi::c_void,
    ) -> i32;
    fn GetFinalPathNameByHandleW(
        handle: *mut std::ffi::c_void,
        path: *mut u16,
        length: u32,
        flags: u32,
    ) -> u32;
}

#[cfg(windows)]
mod identity;
mod storage;
