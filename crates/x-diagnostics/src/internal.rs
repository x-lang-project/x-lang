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
