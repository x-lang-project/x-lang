use inkwell::attributes::AttributeLoc;
use inkwell::context::Context;
use inkwell::targets::{TargetData, TargetMachine, TargetTriple};

use crate::frontend;
use crate::{CHERI_RISCV64_DATALAYOUT, CHERI_RISCV64_PURECAP_PROFILE, CHERI_RISCV64_TRIPLE};
use x_codegen::{CodeGeneration, CodeGenerationOptions, initialize_target_for_triple};
use x_diagnostics::{CompilationFailure, StageDiagnostic};

#[derive(Clone, Copy, PartialEq, Eq)]
enum TargetProfile {
    Native,
    CheriPurecap,
}

pub(super) struct CompilationTarget {
    triple: TargetTriple,
    profile: TargetProfile,
}

impl CompilationTarget {
    pub(super) fn new(requested: Option<&str>) -> Self {
        let profile = if requested == Some(CHERI_RISCV64_PURECAP_PROFILE) {
            TargetProfile::CheriPurecap
        } else {
            TargetProfile::Native
        };

        let effective_triple = match profile {
            TargetProfile::CheriPurecap => Some(CHERI_RISCV64_TRIPLE),
            TargetProfile::Native => requested,
        };

        Self {
            triple: effective_triple
                .map(TargetTriple::create)
                .unwrap_or_else(TargetMachine::get_default_triple),
            profile,
        }
    }

    pub(super) fn is_cheri_purecap(&self) -> bool {
        self.profile == TargetProfile::CheriPurecap
    }

    pub(super) fn configure_stack_probe(&self, codegen: &CodeGeneration<'_>) {
        use x_utils::target::TinyCcWindows;

        if self.triple.as_str().to_bytes() != TinyCcWindows::TRIPLE.as_bytes() {
            return;
        }
        // LLVM probes without adjusting rsp; TinyCC's __chkstk adjusts it.
        let attribute = codegen
            .context
            .create_string_attribute("probe-stack", TinyCcWindows::STACK_PROBE);

        for function in codegen.module.get_functions() {
            if function.count_basic_blocks() != 0 {
                function.add_attribute(AttributeLoc::Function, attribute);
            }
        }
    }

    pub(super) fn prepare_codegen<'ctx>(
        &self,
        context: &'ctx Context,
        optimisation_level: u8,
        input: &frontend::SourceInput<'_>,
    ) -> Result<(CodeGeneration<'ctx>, Option<TargetMachine>), CompilationFailure> {
        let options = match self.profile {
            TargetProfile::CheriPurecap => CodeGenerationOptions::cheri_purecap(),
            TargetProfile::Native => CodeGenerationOptions::default(),
        };

        let level = x_codegen::llvm_optimization_level(optimisation_level);
        let (codegen, machine) = match self.profile {
            TargetProfile::Native => {
                initialize_target_for_triple(&self.triple).map_err(|error| {
                    CompilationFailure::stage(
                        input.text,
                        input.path,
                        StageDiagnostic::LLVM_TARGET_INITIALIZATION,
                        error,
                    )
                })?;
                let machine = x_codegen::create_target_machine_for_initialized_target(
                    &self.triple,
                    optimisation_level,
                )
                .map_err(|error| {
                    CompilationFailure::stage(
                        input.text,
                        input.path,
                        StageDiagnostic::LLVM_TARGET_MACHINE_CREATION,
                        error,
                    )
                })?;
                let codegen =
                    CodeGeneration::with_target_machine(context, "main", level, options, &machine);

                (codegen, Some(machine))
            }
            TargetProfile::CheriPurecap => {
                let codegen = CodeGeneration::with_options(context, "main", level, options)
                    .map_err(|error| {
                        CompilationFailure::stage(
                            input.text,
                            input.path,
                            StageDiagnostic::LLVM_INITIALIZATION,
                            error,
                        )
                    })?;
                initialize_target_for_triple(&self.triple).map_err(|error| {
                    CompilationFailure::stage(
                        input.text,
                        input.path,
                        StageDiagnostic::LLVM_TARGET_INITIALIZATION,
                        error,
                    )
                })?;
                codegen.module.set_data_layout(
                    &TargetData::create(CHERI_RISCV64_DATALAYOUT).get_data_layout(),
                );

                (codegen, None)
            }
        };

        codegen.module.set_triple(&self.triple);

        Ok((codegen, machine))
    }
}
