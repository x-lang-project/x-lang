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

pub(super) fn file_identity_bytes(candidate: &Path) -> Option<Vec<u8>> {
    serde_json::to_vec(&file_identity(candidate).ok()?).ok()
}

pub(super) fn identity_bytes(candidate: &Path) -> Option<Vec<u8>> {
    serde_json::to_vec(&executable_identity(candidate).ok()?).ok()
}

pub(super) fn version(candidate: &Path, query: impl FnOnce() -> Option<String>) -> Option<String> {
    let root = env::var_os("X_RUNTIME_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("x-lang-runtime-cache-v1"))
        .join("toolchain-versions-v1");

    cached_version(candidate, &root, query)
}

fn cached_version(
    candidate: &Path,
    root: &Path,
    query: impl FnOnce() -> Option<String>,
) -> Option<String> {
    let Some(identity) = executable_identity(candidate).ok() else {
        return query();
    };

    let Ok(key) = serde_json::to_vec(&identity) else {
        return query();
    };

    let hash = Sha256::digest(key)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let path = root.join(format!("{hash}.json"));

    if let Some(entry) = fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Entry>(&bytes).ok())
        .filter(|entry| {
            entry.identity == identity
                && clang_is_pinned(&entry.version)
                && executable_identity(candidate).ok().as_ref() == Some(&identity)
        })
    {
        return Some(entry.version);
    }

    let version = query()?;

    if clang_is_pinned(&version) && executable_identity(candidate).ok().as_ref() == Some(&identity)
    {
        let entry = Entry {
            identity,
            version: version.clone(),
        };
        let _ = save_entry(root, &path, &entry);
    }

    Some(version)
}

#[cfg(windows)]
fn executable_identity(candidate: &Path) -> io::Result<Identity> {
    let mut file = File::open(candidate)?;
    let path = resolved_file_path(&file)?;
    let mut magic = [0; 2];
    file.read_exact(&mut magic)?;
    if magic != *b"MZ"
        || path
            .extension()
            .is_none_or(|extension| !extension.eq_ignore_ascii_case("exe"))
    {
        return Err(io::Error::other(
            "version cache requires a native executable",
        ));
    }

    identity_from_file(path, &file)
}

#[cfg(windows)]
fn file_identity(candidate: &Path) -> io::Result<Identity> {
    use std::os::windows::fs::OpenOptionsExt;

    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(0x02000000)
        .open(candidate)?;

    identity_from_file(resolved_file_path(&file)?, &file)
}

#[cfg(windows)]
fn identity_from_file(path: PathBuf, file: &File) -> io::Result<Identity> {
    use std::os::windows::io::AsRawHandle;

    let mut info = FileInformation::default();
    let mut basic = BasicInformation::default();
    // SAFETY: the buffers match the Windows ABI and file owns the handle throughout.
    unsafe {
        if GetFileInformationByHandle(file.as_raw_handle(), &mut info) == 0
            || GetFileInformationByHandleEx(
                file.as_raw_handle(),
                0,
                (&mut basic as *mut BasicInformation).cast(),
                std::mem::size_of::<BasicInformation>() as u32,
            ) == 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    if basic.changed == 0 || info.file_identifier_high == 0 && info.file_identifier_low == 0 {
        return Err(io::Error::other(
            "filesystem does not expose compiler change identity",
        ));
    }

    Ok(Identity {
        path,
        size: (u64::from(info.size_high) << 32) | u64::from(info.size_low),
        volume: info.volume,
        file_identifier: (u64::from(info.file_identifier_high) << 32)
            | u64::from(info.file_identifier_low),
        modified: basic.modified,
        changed: basic.changed,
        usn: file_usn(file)?,
    })
}

#[cfg(not(windows))]
fn executable_identity(_: &Path) -> io::Result<Identity> {
    Err(io::Error::other(
        "native version cache is supported on Windows",
    ))
}

#[cfg(not(windows))]
fn file_identity(_: &Path) -> io::Result<Identity> {
    Err(io::Error::other(
        "native file identity cache is supported on Windows",
    ))
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
