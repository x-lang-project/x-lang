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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Stamp {
    path: PathBuf,
    modified_nanoseconds: u128,
    len: u64,
    directory: bool,
    identity: Option<Vec<u8>>,
}

mod dependencies;

mod build;

mod policy;
