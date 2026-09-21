use std::env;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use x_diagnostics::{CompilationFailure, LinkDiagnostic};
use x_utils::target::TinyCcWindows;

mod library;

// TinyCC has process-global state; keep states and callbacks inside this lock.
