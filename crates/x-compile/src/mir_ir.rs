use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::TargetMachine;

use self::target::CompilationTarget;
use crate::{IrCompileOptions, cheri_subset, frontend};
use x_codegen::{CodeGenerationError, ObjectEmissionOptions, OptimizationPolicy};
use x_diagnostics::{CompilationFailure, SourceIdentifier, source_span_for_diagnostic};
use x_mir::MirProgram;

mod target;

pub(crate) struct GeneratedMirIr<'ctx> {
    pub ir: String,
    representation: IrRepresentation<'ctx>,
}

enum IrRepresentation<'ctx> {
    Native {
        module: Module<'ctx>,
        machine: TargetMachine,
        optimisation_level: u8,
    },
    Text {
        context: &'ctx Context,
    },
}

impl<'ctx> GeneratedMirIr<'ctx> {
    pub(crate) fn take_module_for_jit(self) -> Result<Module<'ctx>, inkwell::support::LLVMString> {
        match self.representation {
            IrRepresentation::Native { module, .. } => Ok(module),
            IrRepresentation::Text { context } => context.create_module_from_ir(
                inkwell::memory_buffer::MemoryBuffer::create_from_memory_range_copy(
                    self.ir.as_bytes(),
                    "jit",
                ),
            ),
        }
    }

    pub fn write_to_object(
        &self,
        options: ObjectEmissionOptions<'_>,
    ) -> Result<(), CodeGenerationError> {
        match &self.representation {
            IrRepresentation::Native {
                module,
                machine,
                optimisation_level,
            } if *optimisation_level == options.optimisation_level
                && machine.get_triple().as_str() == options.triple.as_str() =>
            {
                x_codegen::write_module_to_object_with_target_machine(
                    module,
                    machine,
                    options.output,
                )
            }
            IrRepresentation::Native { module, .. } => {
                x_codegen::write_module_to_object(module, options)
            }
            IrRepresentation::Text { context } => {
                x_codegen::write_ir_to_object(context, &self.ir, options)
            }
        }
    }
}
