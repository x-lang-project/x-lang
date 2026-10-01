use std::{collections::BTreeMap, env, fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{LinkOptions, NativeToolchain, executable_cache};
use x_diagnostics::{CliError, CompilationFailure};

const BRIDGE: &str = "\n#include <stdarg.h>\nint x_jit_printf(const char *format, ...) { va_list args; va_start(args, format); int result = vprintf(format, args); va_end(args); return result; }\nvoid x_jit_flush(void) { fflush(NULL); }\n";
#[derive(Serialize, Deserialize)]
struct Entry {
    path: PathBuf,
    digest: String,
    files: BTreeMap<PathBuf, Option<Vec<u8>>>,
}
fn error(message: impl ToString) -> CompilationFailure {
    CliError::operational(message.to_string()).into_failure()
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
