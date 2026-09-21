use std::fmt;

use std::env;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use library::Library;

use x_diagnostics::{CompilationFailure, LinkDiagnostic};
use x_utils::target::TinyCcWindows;

mod library;

// TinyCC has process-global state; keep states and callbacks inside this lock.

pub(super) fn failure(detail: impl fmt::Display) -> CompilationFailure {
    LinkDiagnostic::TinyCcFailure.into_failure(detail)
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
