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
impl fmt::Display for CUnsupportedReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.detail)
    }
}
#[derive(Debug, Clone)]
pub enum CBackendOutput {
    Supported(CArtifact),
    Unsupported(CUnsupportedReason),
}

pub fn emit_checked_mir(program: &MirProgram) -> Result<CBackendOutput, CompilationFailure> {
    let start = Instant::now();
    program
        .validate()
        .map_err(|failure| InternalDiagnostic::InvalidMir.into_failure(failure))?;

    match emitter::emit(program) {
        Ok(source) => {
            let metrics = CEmissionMetrics {
                elapsed_seconds: start.elapsed().as_secs_f64(),
                source_bytes: source.len(),
                source_lines: source.lines().count(),
            };

            Ok(CBackendOutput::Supported(CArtifact { source, metrics }))
        }
        Err(error::EmitError::Unsupported(detail)) => {
            Ok(CBackendOutput::Unsupported(CUnsupportedReason { detail }))
        }
        Err(error::EmitError::Internal(detail)) => {
            Err(InternalDiagnostic::CEmissionInvariant.into_failure(detail))
        }
    }
}
