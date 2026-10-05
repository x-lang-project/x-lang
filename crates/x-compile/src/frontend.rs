use std::path::Path;

use crate::{DebugArtifact, SourceCatalog};
use x_ast::{Program, SourceSpan};
use x_diagnostics::contract_messages::ContractValidationPhase;
use x_diagnostics::{
    CompilationFailure, Diagnostic, SourceIdentifier, Span, source_span_for_diagnostic,
};
use x_mir::MirProgram;

mod checks;
mod lowering;

pub(crate) struct SourceInput<'a> {
    pub text: &'a str,
    pub path: &'a Path,
}

pub(crate) struct SourceContext<'a> {
    pub input: SourceInput<'a>,
    pub catalog: Option<&'a mut SourceCatalog>,
}

impl SourceContext<'_> {
    fn source_for(&self, span: Option<SourceSpan>) -> (&Path, &str) {
        self.catalog
            .as_deref()
            .and_then(|catalog| catalog.source_for(span))
            .unwrap_or((self.input.path, self.input.text))
    }

    fn diagnostic_failure(
        &self,
        origin: Option<SourceSpan>,
        diagnostic: impl FnOnce(Span) -> Diagnostic,
    ) -> CompilationFailure {
        let (path, source) = self.source_for(origin);
        let span = source_span_for_diagnostic(SourceIdentifier(0), source);

        CompilationFailure::diagnostic(source, path, diagnostic(span))
    }
}

pub(crate) struct FrontendOptions {
    pub no_borrow_check: bool,
    pub show_parse: bool,
    pub contract_phase: ContractValidationPhase,
}

impl Default for FrontendOptions {
    fn default() -> Self {
        Self {
            no_borrow_check: false,
            show_parse: false,
            contract_phase: ContractValidationPhase::Lowering,
        }
    }
}

pub(crate) struct CheckedFrontend {
    pub mir: MirProgram,
    pub debug_artifacts: Vec<DebugArtifact>,
}
