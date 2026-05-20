use std::fmt;

use crate::{CompilationFailure, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InternalDiagnostic {
    SourceDeclarationMismatch,
    SourceCapacityExceeded,
    ResolvedModuleMissing,
    InvalidHir,
    InvalidMir,
    CEmissionInvariant,
    ManifestExecutableMissing,
}

impl InternalDiagnostic {
    pub const fn stage(self) -> &'static str {
        match self {
            Self::SourceDeclarationMismatch | Self::SourceCapacityExceeded => "source registration",
            Self::ResolvedModuleMissing => "module assembly",
            Self::InvalidHir => "HIR validation",
            Self::InvalidMir => "MIR validation",
            Self::CEmissionInvariant => "C emission",
            Self::ManifestExecutableMissing => "manifest build",
        }
    }
}
