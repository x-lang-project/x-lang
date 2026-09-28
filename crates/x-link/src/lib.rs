use std::{fs, path::Path};

use x_logging::info;
use x_runtime::write_runtime_file;

#[derive(Debug, Clone, Default)]
pub struct LinkOptions<'a> {
    pub libs: &'a [String],
    pub lib_paths: &'a [String],
    pub include_paths: &'a [String],
    pub optimisation_level: u8,
    pub target_triple: Option<&'a str>,
    pub freestanding: bool,
    pub passthrough_arguments: &'a [String],
    pub reproducible: bool,
    pub gc_sections: bool,
}

pub mod jit_runtime;

mod runtime_cache;
pub mod tinycc;
mod toolchain;
#[cfg(windows)]
mod windows_file;
pub use toolchain::{NativeToolchain, clang_program};
