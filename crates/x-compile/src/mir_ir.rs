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

pub(crate) fn generate_mir_ir<'ctx>(
    context: &'ctx Context,
    mir_program: &MirProgram,
    input: frontend::SourceInput<'_>,
    options: IrCompileOptions<'_>,
) -> Result<GeneratedMirIr<'ctx>, CompilationFailure> {
    let target = CompilationTarget::new(options.target_triple);
    let (mut codegen, machine) =
        target.prepare_codegen(context, options.optimisation_level, &input)?;
    let cheri_purecap = target.is_cheri_purecap();
    let codegen_failure = |error: CodeGenerationError| {
        let span = source_span_for_diagnostic(SourceIdentifier(0), input.text);

        CompilationFailure::diagnostic(input.text, input.path, error.into_diagnostic(span))
    };

    if cheri_purecap {
        cheri_subset::validate(mir_program)?;
    }

    codegen.generate(mir_program).map_err(codegen_failure)?;
    target.configure_stack_probe(&codegen);

    if let Some(machine) = &machine {
        codegen
            .optimize_module_with_target_machine_with_policy(
                options.optimisation_level,
                machine,
                OptimizationPolicy {
                    dce: options.dce,
                    inline_budget: options.inline_budget,
                },
            )
            .map_err(codegen_failure)?;
    }

    let mut ir = codegen.get_ir();

    if cheri_purecap {
        ir = ir
            .replace(
                "@__x_cheri_bounds_set_exact",
                "@llvm.cheri.cap.bounds.set.exact.i64",
            )
            .replace("@__x_cheri_perms_and", "@llvm.cheri.cap.perms.and.i64");
    }

    let needs_rewrite = cheri_purecap;

    let representation = match (needs_rewrite, machine) {
        (false, Some(machine)) => IrRepresentation::Native {
            module: codegen.module,
            machine,
            optimisation_level: options.optimisation_level,
        },
        _ => IrRepresentation::Text { context },
    };

    Ok(GeneratedMirIr { ir, representation })
}
