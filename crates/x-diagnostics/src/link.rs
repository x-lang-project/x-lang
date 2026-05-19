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
