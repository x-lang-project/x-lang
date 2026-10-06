use std::path::Path;

use inkwell::context::Context;

use crate::{CHERI_RISCV64_PURECAP_PROFILE, DebugArtifact, GeneratedIr, IrCompileOptions};
use crate::{SourceMirOptions, compile_source_to_mir, frontend, mir_ir};
use x_diagnostics::CompilationFailure;

pub fn compile_and_generate_ir(
    context: &Context,
    source: &str,
    input_path: &str,
    options: IrCompileOptions<'_>,
) -> Result<GeneratedIr, CompilationFailure> {
    let (generated, debug_artifacts) =
        compile_and_generate_module(context, source, input_path, options)?;

    Ok(GeneratedIr {
        ir: generated.ir,
        debug_artifacts,
    })
}

pub(crate) fn compile_and_generate_module<'ctx>(
    context: &'ctx Context,
    source: &str,
    input_path: &str,
    options: IrCompileOptions<'_>,
) -> Result<(mir_ir::GeneratedMirIr<'ctx>, Vec<DebugArtifact>), CompilationFailure> {
    let checked = compile_source_to_mir(
        source,
        Path::new(input_path),
        SourceMirOptions {
            show_parse: options.show_parse,
            optimizer_diagnostics: options.optimizer_diagnostics,
        },
    )?;

    let ir = mir_ir::generate_mir_ir(
        context,
        &checked.mir,
        frontend::SourceInput {
            text: source,
            path: Path::new(input_path),
        },
        options,
    )?;

    Ok((ir, checked.debug_artifacts))
}

pub fn compile_source_to_ir(
    source: &str,
    input_path: &str,
    options: IrCompileOptions<'_>,
) -> Result<String, CompilationFailure> {
    let context = Context::create();

    compile_and_generate_ir(&context, source, input_path, options).map(|artifact| artifact.ir)
}

pub fn compile_source_to_cheri_riscv64_ir(
    source: &str,
    input_path: &str,
) -> Result<String, CompilationFailure> {
    compile_source_to_ir(
        source,
        input_path,
        IrCompileOptions {
            target_triple: Some(CHERI_RISCV64_PURECAP_PROFILE),
            ..Default::default()
        },
    )
}
