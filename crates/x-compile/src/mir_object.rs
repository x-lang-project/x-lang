use std::time::Instant;
use std::{fs, path::Path};

use inkwell::targets;

use crate::IrCompileOptions;

use x_codegen::CodeGenerationError;
use x_codegen::ObjectEmissionOptions;
use x_diagnostics::CompilationFailure;
use x_diagnostics::SourceIdentifier;
use x_diagnostics::Span;
use x_diagnostics::contract_messages;
use x_mir::MirProgram;
use x_utils::artifacts::ArtifactPaths;

#[derive(Clone, Copy, Debug, Default)]
pub struct MirObjectOptions<'a> {
    pub optimisation_level: u8,
    pub target: Option<&'a str>,
    pub save_ir: bool,
}

fn render_manifest_codegen_error(
    module_name: &str,
    error: CodeGenerationError,
) -> CompilationFailure {
    let diagnostic = error.into_diagnostic(Span {
        source: SourceIdentifier(0),
        start: 0,
        end: 1,
    });

    CompilationFailure::diagnostic("", Path::new(module_name), diagnostic)
        .with_note(contract_messages::module_object_emission_note(module_name))
}
