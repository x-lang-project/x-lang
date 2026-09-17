use std::path::Path;

use inkwell::targets::{FileType, TargetMachine};
use inkwell::{context, memory_buffer, module, targets};

use super::target::create_target_machine;
use super::verification::{VerificationStage, verify_module};
use crate::errors::CodeGenerationError;
use x_diagnostics::backend_messages;

#[derive(Clone, Copy)]
pub struct ObjectEmissionOptions<'a> {
    pub triple: &'a targets::TargetTriple,
    pub optimisation_level: u8,
    pub output: &'a Path,
}

pub fn write_module_to_object(
    module: &module::Module<'_>,
    options: ObjectEmissionOptions<'_>,
) -> Result<(), CodeGenerationError> {
    let ObjectEmissionOptions {
        triple,
        optimisation_level,
        output,
    } = options;

    module.set_triple(triple);
    let machine = create_target_machine(triple, optimisation_level)?;

    write_module_to_object_with_target_machine(module, &machine, output)
}

pub fn write_module_to_object_with_target_machine(
    module: &module::Module<'_>,
    machine: &TargetMachine,
    output: &Path,
) -> Result<(), CodeGenerationError> {
    module.set_triple(&machine.get_triple());
    module.set_data_layout(&machine.get_target_data().get_data_layout());
    verify_module(module, VerificationStage::BeforeObjectEmission)?;

    machine
        .write_to_file(module, FileType::Object, output)
        .map_err(|error| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::llvm_object_emission_to_value_failed_error(
                    output.display(),
                    error,
                ),
            )
        })
}

pub fn write_ir_to_object(
    context: &context::Context,
    ir: &str,
    options: ObjectEmissionOptions<'_>,
) -> Result<(), CodeGenerationError> {
    let mut terminated_ir = Vec::with_capacity(ir.len() + 1);
    terminated_ir.extend_from_slice(ir.as_bytes());
    terminated_ir.push(0);
    let buffer = memory_buffer::MemoryBuffer::create_from_memory_range_copy(
        &terminated_ir,
        "x-validated-ir",
    );
    let module = context.create_module_from_ir(buffer).map_err(|error| {
        CodeGenerationError::LlvmVerificationFailure(
            backend_messages::generated_textual_llvm_ir_could_not_be_parsed_error(error),
        )
    })?;

    write_module_to_object(&module, options)
}
