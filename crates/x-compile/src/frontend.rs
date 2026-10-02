use std::path::Path;

use crate::{DebugArtifact, SourceCatalog};
use x_ast::{Program, SourceSpan};
use x_diagnostics::contract_messages::ContractValidationPhase;
use x_diagnostics::{
    CompilationFailure, Diagnostic, SourceIdentifier, Span, source_span_for_diagnostic,
};
use x_mir::MirProgram;

pub(crate) struct SourceInput<'a> {
    pub text: &'a str,
    pub path: &'a Path,
}
