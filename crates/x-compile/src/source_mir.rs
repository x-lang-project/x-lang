use std::path::Path;

use crate::DebugArtifact;

use x_diagnostics::CompilationFailure;
use x_diagnostics::contract_messages::ContractValidationPhase;
use x_mir::MirProgram;
use x_parser::parse;

#[derive(Clone, Copy, Debug, Default)]
pub struct SourceMirOptions {
    pub show_parse: bool,
    pub optimizer_diagnostics: bool,
}

#[derive(Debug)]
pub struct SourceMirArtifact {
    pub mir: MirProgram,
    pub debug_artifacts: Vec<DebugArtifact>,
}
