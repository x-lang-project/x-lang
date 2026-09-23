use std::{collections::BTreeMap, env, fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::NativeToolchain;
use x_diagnostics::{CliError, CompilationFailure};

fn error(message: impl ToString) -> CompilationFailure {
    CliError::operational(message.to_string()).into_failure()
}
