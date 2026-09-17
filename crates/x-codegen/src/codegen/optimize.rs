use inkwell::targets::{TargetMachine, TargetTriple};

use super::CodeGeneration;
use crate::errors::CodeGenerationError;
use pipeline::OptimizationPipeline;

mod emission;
mod pipeline;
mod target;
mod verification;

pub use emission::{
    ObjectEmissionOptions, write_ir_to_object, write_module_to_object,
    write_module_to_object_with_target_machine,
};
pub use pipeline::OptimizationPolicy;
pub use target::{
    create_target_machine, create_target_machine_for_initialized_target,
    initialize_target_for_triple, llvm_optimization_level,
};

impl CodeGeneration<'_> {
    pub fn optimize_module(&self, optimisation_level: u8) -> Result<(), CodeGenerationError> {
        self.optimize_module_for_triple(optimisation_level, &TargetMachine::get_default_triple())
    }

    pub fn optimize_module_for_triple(
        &self,
        optimisation_level: u8,
        triple: &TargetTriple,
    ) -> Result<(), CodeGenerationError> {
        self.optimize_module_for_triple_with_policy(
            optimisation_level,
            triple,
            OptimizationPolicy::default(),
        )
    }

    pub fn optimize_module_for_triple_with_policy(
        &self,
        optimisation_level: u8,
        triple: &TargetTriple,
        policy: OptimizationPolicy,
    ) -> Result<(), CodeGenerationError> {
        let Some(pipeline) =
            OptimizationPipeline::prepare(&self.module, optimisation_level, policy)?
        else {
            return Ok(());
        };

        let machine = create_target_machine(triple, optimisation_level)?;

        pipeline.run(&self.module, &machine)
    }

    pub fn optimize_module_with_target_machine_with_policy(
        &self,
        optimisation_level: u8,
        machine: &TargetMachine,
        policy: OptimizationPolicy,
    ) -> Result<(), CodeGenerationError> {
        let Some(pipeline) =
            OptimizationPipeline::prepare(&self.module, optimisation_level, policy)?
        else {
            return Ok(());
        };

        pipeline.run(&self.module, machine)
    }
}
