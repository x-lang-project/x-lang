use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{LinkOptions, NativeToolchain};
use build::*;
use dependencies::*;
use policy::*;

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
