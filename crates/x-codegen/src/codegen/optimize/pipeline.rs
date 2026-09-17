use inkwell::module::Module;
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::TargetMachine;

use super::verification::{VerificationStage, verify_module};
use crate::errors::CodeGenerationError;
use x_diagnostics::backend_messages;

#[derive(Debug, Clone, Copy, Default)]
pub struct OptimizationPolicy {
    pub dce: bool,
    pub inline_budget: u32,
}

pub(super) struct OptimizationPipeline {
    passes: String,
}

impl OptimizationPipeline {
    pub(super) fn prepare(
        module: &Module<'_>,
        optimisation_level: u8,
        policy: OptimizationPolicy,
    ) -> Result<Option<Self>, CodeGenerationError> {
        verify_module(module, VerificationStage::BeforeOptimization)?;

        Ok(optimization_passes(optimisation_level, policy).map(|passes| Self { passes }))
    }

    pub(super) fn run(
        self,
        module: &Module<'_>,
        machine: &TargetMachine,
    ) -> Result<(), CodeGenerationError> {
        module
            .run_passes(&self.passes, machine, PassBuilderOptions::create())
            .map_err(|error| {
                CodeGenerationError::LlvmInternalError(
                    backend_messages::llvm_optimisation_failed_e(error),
                )
            })?;

        verify_module(module, VerificationStage::AfterOptimization)
    }
}

pub(super) fn optimization_passes(
    optimisation_level: u8,
    policy: OptimizationPolicy,
) -> Option<String> {
    if optimisation_level == 0 {
        return Some("function(mem2reg)".to_string());
    }

    let mut passes = match optimisation_level {
        1 => "function(mem2reg),default<O1>",
        2 => "function(mem2reg),default<O2>",
        3 => "function(mem2reg),default<O3>",
        _ => return None,
    }
    .to_string();

    if policy.dce {
        passes.push_str(",globaldce");
    }

    if policy.inline_budget > 0 && optimisation_level < 3 {
        passes.push_str(",cgscc(inline)");
    }

    Some(passes)
}
