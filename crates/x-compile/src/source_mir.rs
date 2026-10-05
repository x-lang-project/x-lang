use std::path::Path;

use crate::DebugArtifact;
use crate::frontend;
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

pub fn check_source(source: &str, input_path: &Path) -> Result<(), CompilationFailure> {
    let program = parse(source).map_err(|error| error.with_source(source, input_path))?;

    frontend::check_and_lower(
        program,
        Default::default(),
        frontend::SourceContext {
            input: frontend::SourceInput {
                text: source,
                path: input_path,
            },
            catalog: None,
        },
    )?;

    Ok(())
}

pub fn compile_source_to_mir(
    source: &str,
    input_path: &Path,
    options: SourceMirOptions,
) -> Result<SourceMirArtifact, CompilationFailure> {
    let program = parse(source).map_err(|error| error.with_source(source, input_path))?;
    let checked = frontend::check_and_lower(
        program,
        frontend::FrontendOptions {
            show_parse: options.show_parse,
            contract_phase: if options.optimizer_diagnostics {
                ContractValidationPhase::Optimisation
            } else {
                ContractValidationPhase::Lowering
            },
            ..Default::default()
        },
        frontend::SourceContext {
            input: frontend::SourceInput {
                text: source,
                path: input_path,
            },
            catalog: None,
        },
    )?;

    Ok(SourceMirArtifact {
        mir: checked.mir,
        debug_artifacts: checked.debug_artifacts,
    })
}
