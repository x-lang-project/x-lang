use std::fmt;

use crate::{CompilationFailure, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkDiagnostic {
    ObjectRead,
    ObjectNotFile,
    ObjectEmpty,
    ObjectResolve,
    ClangMissing,
    ConfiguredClangUnavailable,
    ConfiguredClangVersion,
    ClangLaunch,
    OutputMissing,
    ClangFailure,
    TinyCcFailure,
    ObjectInputsMissing,
    RuntimeSourceWrite,
}

impl LinkDiagnostic {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ObjectRead
            | Self::ObjectNotFile
            | Self::ObjectEmpty
            | Self::ObjectResolve
            | Self::ClangMissing
            | Self::ClangLaunch
            | Self::ClangFailure
            | Self::TinyCcFailure
            | Self::ObjectInputsMissing => "L0001",
            Self::ConfiguredClangUnavailable => "L0002",
            Self::ConfiguredClangVersion => "L0003",
            Self::OutputMissing => "L0004",
            Self::RuntimeSourceWrite => "L0005",
        }
    }
}
