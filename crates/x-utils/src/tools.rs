use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::artifacts::EXECUTABLE_SUFFIX;

mod llvm {
    include!("../build/llvm_policy.rs");
}
pub use llvm::{LLVM_PREFIX_ENV, LLVM_VERSION, clang_is_pinned};

pub fn executable_name(name: &str) -> String {
    if cfg!(windows) && !name.ends_with(EXECUTABLE_SUFFIX) {
        format!("{name}{EXECUTABLE_SUFFIX}")
    } else {
        name.into()
    }
}
