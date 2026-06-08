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

pub fn find_on_path(program: &str) -> Option<PathBuf> {
    find_in_path(program, &env::var_os("PATH")?)
}

pub fn find_in_path(program: &str, path: &OsStr) -> Option<PathBuf> {
    let configured = Path::new(program);

    if configured.is_absolute() || configured.components().count() > 1 {
        return configured.is_file().then(|| configured.to_path_buf());
    }

    let names = [program.to_string(), executable_name(program)];

    for directory in env::split_paths(path) {
        for name in &names {
            let candidate = directory.join(name);

            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

pub fn resolve_program(program: &str) -> PathBuf {
    find_on_path(program).unwrap_or_else(|| PathBuf::from(program))
}
