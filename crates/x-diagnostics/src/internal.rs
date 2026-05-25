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
    pub fn missing_manifest_executable() -> CompilationFailure {
        Self::ManifestExecutableMissing.into_failure("no executable")
    }

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

    pub fn into_failure(self, detail: impl fmt::Display) -> CompilationFailure {
        Error::new(detail.to_string())
            .with_code(self.code())
            .into_internal_failure(self.stage())
    }
}
