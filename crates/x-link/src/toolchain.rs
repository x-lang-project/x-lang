use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use x_diagnostics::CompilationFailure;
use x_diagnostics::link_messages;
use x_utils::tools::LLVM_VERSION as REQUIRED_CLANG_VERSION;
use x_utils::tools::{LLVM_PREFIX_ENV, clang_is_pinned};

mod discovery_state;
mod version_cache;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigurationSource {
    ExplicitCompiler,
    LlvmPrefix,
}

impl ConfigurationSource {
    const fn environment_name(self) -> &'static str {
        match self {
            Self::ExplicitCompiler => "X_CLANG",
            Self::LlvmPrefix => LLVM_PREFIX_ENV,
        }
    }
}

fn clang_name() -> &'static str {
    if cfg!(windows) { "clang.exe" } else { "clang" }
}
