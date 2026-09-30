use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{LinkOptions, NativeToolchain};
use x_diagnostics::{CompilationFailure, link_messages};

// Parse Clang's quoted -### arguments without invoking a shell.

mod plans;
pub(crate) use plans::*;
