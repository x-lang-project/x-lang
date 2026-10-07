use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use inkwell::targets::{TargetMachine, TargetTriple};

use crate::{
    CHERI_RISCV64_PURECAP_PROFILE, CompilationFailure, CompileArtifact, GeneratedIr,
    IrCompileOptions, ObjectCompileOptions, compile_and_generate_module,
};
use x_codegen::ObjectEmissionOptions;
use x_diagnostics::{StageDiagnostic, contract_messages};
use x_logging::info;
use x_project::manifest::Manifest;
use x_utils::artifacts::ArtifactPaths;
use x_utils::project::find_project_manifest;

pub fn compile_file_to_object(
    source: &str,
    input_path: &str,
    output_object: &str,
    options: ObjectCompileOptions<'_>,
) -> Result<CompileArtifact, CompilationFailure> {
    let ObjectCompileOptions {
        ir: ir_options,
        save_ir,
    } = options;
    let target_triple = ir_options.target_triple;
    let optimisation_level = ir_options.optimisation_level;

    if target_triple == Some(CHERI_RISCV64_PURECAP_PROFILE) {
        return Err(contract_messages::cheri_certified_build_required());
    }

    let compile_start = Instant::now();

    let resolved_triple_owned = if target_triple.is_none() {
        find_target_triple_from_manifest(input_path)?
    } else {
        None
    };

    let effective_triple = target_triple.or(resolved_triple_owned.as_deref());

    let context = crate::object_context(ir_options.discard_value_names && !save_ir);

    let (generated, debug_artifacts) = compile_and_generate_module(
        &context,
        source,
        input_path,
        IrCompileOptions {
            target_triple: effective_triple,
            ..ir_options
        },
    )?;

    let final_object_path = PathBuf::from(output_object);

    if let Some(parent) = final_object_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| contract_messages::object_output_directory(parent.display(), error))?;
    }

    let triple = effective_triple
        .map(TargetTriple::create)
        .unwrap_or_else(TargetMachine::get_default_triple);
    let emission_options = ObjectEmissionOptions {
        triple: &triple,
        optimisation_level,
        output: &final_object_path,
    };

    generated
        .write_to_object(emission_options)
        .map_err(|error| {
            CompilationFailure::stage(
                source,
                input_path,
                StageDiagnostic::LLVM_OBJECT_EMISSION,
                error,
            )
        })?;

    let ir = generated.ir;

    if save_ir {
        archive_ir_copy(input_path, optimisation_level, &ir)?;
    }

    let elapsed_seconds = compile_start.elapsed().as_secs_f64();
    info!(
        "Compiled {input_path} -> {} in {elapsed_seconds:.6} seconds",
        final_object_path.display()
    );

    Ok(CompileArtifact::new(
        final_object_path,
        triple.as_str().to_string_lossy().into_owned(),
        elapsed_seconds,
        GeneratedIr {
            ir,
            debug_artifacts,
        },
    ))
}

fn find_target_triple_from_manifest(
    input_path: &str,
) -> Result<Option<String>, CompilationFailure> {
    let Some(path) = find_project_manifest(Path::new(input_path)) else {
        return Ok(None);
    };

    let manifest = Manifest::load(&path)?;

    Ok(manifest.target().map(String::from))
}

fn archive_ir_copy(
    input_path: &str,
    optimisation_level: u8,
    ir_content: &str,
) -> Result<(), CompilationFailure> {
    let save_name = ArtifactPaths::new(input_path).archived_ir(optimisation_level);

    fs::write(&save_name, ir_content)
        .map_err(|error| contract_messages::ir_save(save_name.display(), error))?;
    info!("Saved IR to {}", save_name.display());

    Ok(())
}
