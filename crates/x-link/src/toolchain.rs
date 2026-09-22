use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use x_diagnostics::CompilationFailure;
use x_diagnostics::link_messages;
use x_utils::tools::LLVM_VERSION as REQUIRED_CLANG_VERSION;
use x_utils::tools::{LLVM_PREFIX_ENV, clang_is_pinned};

mod version_cache;
