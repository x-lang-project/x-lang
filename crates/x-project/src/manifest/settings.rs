use std::path::{Path, PathBuf};

use x_diagnostics::CompilationFailure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageKind {
    Binary,
    Library,
}

impl PackageKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Binary => "bin",
            Self::Library => "lib",
        }
    }
}
