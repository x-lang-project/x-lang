use std::fmt;

use std::env;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::LinkOutcome;
use crate::{LinkOptions, NativeToolchain};
use library::Library;

use x_diagnostics::{CompilationFailure, LinkDiagnostic};
use x_utils::target::TinyCcWindows;

mod cache;

mod library;
mod prepared;
#[cfg(feature = "tinycc-session")]
pub(super) mod session;

// TinyCC has process-global state; keep states and callbacks inside this lock.
static TINYCC: Mutex<()> = Mutex::new(());

pub(super) fn failure(detail: impl fmt::Display) -> CompilationFailure {
    LinkDiagnostic::TinyCcFailure.into_failure(detail)
}

pub(super) fn supported_options(options: &LinkOptions<'_>) -> bool {
    options.optimisation_level == 0
        && !options.freestanding
        && !options.gc_sections
        && options.libs.is_empty()
        && options.lib_paths.is_empty()
        && options.include_paths.is_empty()
        && options.passthrough_arguments.is_empty()
        && options
            .target_triple
            .is_none_or(|target| target == TinyCcWindows::HOST_TRIPLE)
        && ["CPATH", "C_INCLUDE_PATH", "CPLUS_INCLUDE_PATH"]
            .iter()
            .all(|name| env::var_os(name).is_none_or(|value| value.is_empty()))
        && env::var_os("X_RUNTIME_CACHE_DIR")
            .unwrap_or_else(|| env::temp_dir().into_os_string())
            .is_ascii()
}

pub(super) struct Configuration {
    directory: PathBuf,
    object_type: OutputKind,
    ucrt: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OutputKind {
    Executable,
    Object,
}

impl OutputKind {
    pub(super) const fn abi(self) -> c_int {
        match self {
            Self::Executable => 2,
            Self::Object => 3,
        }
    }
}

pub(super) fn c_path(path: &Path) -> Result<CString, CompilationFailure> {
    // TinyCC's fopen needs DOS/UNC paths instead of extended-length paths.
    let text = path
        .to_str()
        .ok_or_else(|| failure("TinyCC path is not valid Unicode"))?;
    let text = if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        text.strip_prefix(r"\\?\").unwrap_or(text).to_owned()
    };

    CString::new(text).map_err(failure)
}

fn validate_options_abi(header: &str) -> Result<(), CompilationFailure> {
    if header
        .split_whitespace()
        .zip(header.split_whitespace().skip(1))
        .any(|(value_type, name)| value_type == "int" && name.starts_with("tcc_set_options("))
    {
        Ok(())
    } else {
        Err(failure("TinyCC requires int-returning tcc_set_options"))
    }
}
