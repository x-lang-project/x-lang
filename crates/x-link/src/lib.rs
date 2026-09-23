use std::{fs, path::Path};

use x_logging::info;
use x_runtime::write_runtime_file;

pub mod jit_runtime;

pub mod tinycc;
mod toolchain;
#[cfg(windows)]
mod windows_file;
pub use toolchain::{NativeToolchain, clang_program};
