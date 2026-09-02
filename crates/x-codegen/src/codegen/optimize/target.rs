use inkwell::OptimizationLevel;
use inkwell::targets;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};

use crate::errors::CodeGenerationError;
use x_diagnostics::backend_messages;

pub fn create_target_machine(
    triple: &targets::TargetTriple,
    optimisation_level: u8,
) -> Result<TargetMachine, CodeGenerationError> {
    initialize_target_for_triple(triple)?;

    create_target_machine_for_initialized_target(triple, optimisation_level)
}

pub fn create_target_machine_for_initialized_target(
    triple: &targets::TargetTriple,
    optimisation_level: u8,
) -> Result<TargetMachine, CodeGenerationError> {
    let target = Target::from_triple(triple).map_err(|error| {
        CodeGenerationError::LlvmInternalError(
            backend_messages::failed_to_create_target_from_triple_error(error),
        )
    })?;

    target
        .create_target_machine(
            triple,
            "generic",
            "",
            llvm_optimization_level(optimisation_level),
            RelocMode::PIC,
            CodeModel::Default,
        )
        .ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::failed_to_create_llvm_target_machine(),
            )
        })
}

pub fn initialize_target_for_triple(
    triple: &targets::TargetTriple,
) -> Result<(), CodeGenerationError> {
    let text = triple.as_str().to_string_lossy();

    if let Some(spec) = x_utils::target::TargetSpec::parse(&text) {
        match spec.architecture {
            x_utils::target::Architecture::RiscV32 | x_utils::target::Architecture::RiscV64 => {
                Target::initialize_riscv(&InitializationConfig::default());
            }
            x_utils::target::Architecture::X86 | x_utils::target::Architecture::X86_64 => {
                Target::initialize_x86(&InitializationConfig::default());
            }
        }

        return Ok(());
    }

    Err(CodeGenerationError::LlvmInternalError(
        backend_messages::target_text_is_not_enabled_in_this_x_compiler_build(text),
    ))
}

pub fn llvm_optimization_level(optimisation_level: u8) -> OptimizationLevel {
    match optimisation_level {
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        3 => OptimizationLevel::Aggressive,
        _ => OptimizationLevel::None,
    }
}
