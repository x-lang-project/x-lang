use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::linker::RuntimeSource;
use crate::{LinkOptions, NativeToolchain};
use build::*;
use dependencies::*;
use policy::*;

static NEXT_ENTRY: AtomicU64 = AtomicU64::new(0);
const FORMAT: u32 = 4;

struct RuntimeBuild<'a, 'options> {
    toolchain: &'a NativeToolchain,
    source: &'a RuntimeSource,
    options: &'a LinkOptions<'options>,
    key: &'a str,
}

struct TemporaryEntry(PathBuf);

impl Drop for TemporaryEntry {
    fn drop(&mut self) {
        cleanup_entry(&self.0);
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Stamp {
    path: PathBuf,
    modified_nanoseconds: u128,
    len: u64,
    directory: bool,
    identity: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileDigest {
    path: PathBuf,
    digest: String,
    identity: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    format: u32,
    key: String,
    headers: Vec<FileDigest>,
    directories: Vec<Stamp>,
    object_digest: String,
}

fn resolved_compiler(path: &Path) -> io::Result<PathBuf> {
    if path.is_file() {
        return fs::canonicalize(path);
    }

    Err(io::Error::other(
        "runtime cache cannot resolve compiler identity",
    ))
}

pub(crate) fn valid_entry(directory: &Path, key: &str) -> bool {
    let Ok(bytes) = fs::read(directory.join("manifest.json")) else {
        return false;
    };

    let Ok(manifest) = serde_json::from_slice::<Manifest>(&bytes) else {
        return false;
    };

    manifest.format == FORMAT
        && manifest.key == key
        && !manifest.headers.is_empty()
        && !manifest.directories.is_empty()
        && manifest
            .directories
            .iter()
            .all(|saved| stamp(&saved.path).is_ok_and(|current| current == *saved))
        && manifest.headers.iter().all(|saved| match &saved.identity {
            Some(identity) => {
                NativeToolchain::file_identity(&saved.path).as_ref() == Some(identity)
            }
            None => fs::read(&saved.path).is_ok_and(|bytes| digest(&bytes) == saved.digest),
        })
        && fs::read(directory.join("runtime.obj"))
            .is_ok_and(|bytes| !bytes.is_empty() && digest(&bytes) == manifest.object_digest)
}

fn cleanup_entry(path: &Path) {
    // Only these files in the unique directory created by this call are ours.
    for name in ["runtime.obj", "runtime.d", "manifest.json"] {
        let _ = fs::remove_file(path.join(name));
    }

    let _ = fs::remove_dir(path);
}

mod dependencies;

mod build;

mod policy;
