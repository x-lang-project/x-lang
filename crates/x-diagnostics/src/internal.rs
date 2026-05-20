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
    pub const fn code(self) -> &'static str {
        match self {
            Self::SourceDeclarationMismatch => "ICE-SOURCE-0001",
            Self::SourceCapacityExceeded => "ICE-SOURCE-0002",
            Self::ResolvedModuleMissing => "ICE-MODULE-0001",
            Self::InvalidHir => "ICE-H0002",
            Self::InvalidMir => "ICE-M0002",
            Self::CEmissionInvariant => "ICE-CBACKEND-0001",
            Self::ManifestExecutableMissing => "ICE-MANIFEST-0001",
        }
    }

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
