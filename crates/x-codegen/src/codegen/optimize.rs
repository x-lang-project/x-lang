use inkwell::targets::{TargetMachine, TargetTriple};

use super::CodeGeneration;
use crate::errors::CodeGenerationError;

mod target;
mod verification;

pub use target::{
    create_target_machine, create_target_machine_for_initialized_target,
    initialize_target_for_triple, llvm_optimization_level,
};
