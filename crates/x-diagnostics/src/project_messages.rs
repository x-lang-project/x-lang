use std::fmt;

use crate::Error;
use crate::{CompilationFailure, ProjectDiagnostic};

/// Byte and line coordinates reported by the TOML parser.
#[derive(Debug, Clone, Copy)]
pub struct ManifestPosition {
    pub line: usize,
    pub column: usize,
    pub byte: usize,
}

pub struct ProjectError;

mod freestanding;
mod lock;
mod manifest;
mod packages;
mod sources;
