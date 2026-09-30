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

const OUTPUT_TOKEN: &str = "__X_LINK_OUTPUT__";
const INPUT_PREFIX: &str = "__X_LINK_INPUT_";

#[derive(Serialize, Deserialize)]
struct LinkPlan {
    arguments: Vec<String>,
    stamps: BTreeMap<PathBuf, (u64, u128)>,
}

pub(crate) struct LinkRequest<'a> {
    pub(crate) arguments: &'a [String],
    pub(crate) inputs: &'a [String],
    pub(crate) output: &'a str,
    pub(crate) options: &'a LinkOptions<'a>,
}

// Parse Clang's quoted -### arguments without invoking a shell.

mod plans;
pub(crate) use plans::*;
