use std::{fmt, time::Instant};

use x_diagnostics::{CompilationFailure, InternalDiagnostic};
use x_mir::MirProgram;

mod emitter;
mod error;

pub use crate::{CArtifact, CEmissionMetrics};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CUnsupportedReason {
    pub detail: String,
}
