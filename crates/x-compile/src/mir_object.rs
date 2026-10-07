use std::time::Instant;
use std::{fs, path::Path};

use inkwell::targets;

use crate::CompileArtifact;
use crate::GeneratedIr;
use crate::IrCompileOptions;
use crate::frontend;
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

pub fn compile_mir_program_to_object(
    mir_program: &MirProgram,
    project_root: &Path,
    module_name: &str,
    options: MirObjectOptions<'_>,
) -> Result<CompileArtifact, CompilationFailure> {
    let MirObjectOptions {
        optimisation_level,
        target,
        save_ir,
    } = options;

    let started = Instant::now();
    let context = crate::object_context(!save_ir);
    let triple = target
        .map(targets::TargetTriple::create)
        .unwrap_or_else(targets::TargetMachine::get_default_triple);
    let generated = crate::mir_ir::generate_mir_ir(
        &context,
        mir_program,
        frontend::SourceInput {
            text: "",
            path: Path::new(module_name),
        },
        IrCompileOptions {
            optimisation_level,
            target_triple: target,
            ..Default::default()
        },
    )?;

    if save_ir {
        let ir_path = ArtifactPaths::new(project_root.join(module_name)).generated_ir();
        fs::write(&ir_path, &generated.ir).map_err(|error| {
            contract_messages::ir_save_failed_to_save_value_error(ir_path.display(), error)
        })?;
    }

    let object_path = ArtifactPaths::new(project_root.join(module_name)).generated_object();
    let emission_options = ObjectEmissionOptions {
        triple: &triple,
        optimisation_level,
        output: &object_path,
    };

    generated
        .write_to_object(emission_options)
        .map_err(|error| render_manifest_codegen_error(module_name, error))?;

    Ok(CompileArtifact::new(
        object_path,
        triple.as_str().to_string_lossy().into_owned(),
        started.elapsed().as_secs_f64(),
        GeneratedIr {
            ir: generated.ir,
            debug_artifacts: vec![],
        },
    ))
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
