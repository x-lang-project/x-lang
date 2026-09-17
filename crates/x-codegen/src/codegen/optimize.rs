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
