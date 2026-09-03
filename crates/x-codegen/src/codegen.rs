mod optimize;
pub use optimize::create_target_machine;
pub use optimize::create_target_machine_for_initialized_target;
pub use optimize::initialize_target_for_triple;
pub use optimize::llvm_optimization_level;
mod records;

mod types;

use std::{cell::RefCell, collections::HashMap};

use inkwell::targets;
use inkwell::{
    AddressSpace, OptimizationLevel, builder::Builder, context::Context,
    execution_engine::ExecutionEngine, module::Module,
};

use crate::errors::CodeGenerationError;
use x_logging::info;
use x_mir::MirProgram;

pub struct CodeGeneration<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    execution_engine: RefCell<Option<ExecutionEngine<'ctx>>>,
    jit_optimization_level: OptimizationLevel,
    pub(crate) records: records::RecordCatalogue,
    options: CodeGenerationOptions,
    function_symbols: HashMap<String, String>,
    global_symbols: HashMap<String, String>,
    cheri_projection_records: RefCell<Vec<CheriProjectionRecord>>,
    contract_obligations: RefCell<x_mir::ContractObligations>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheriPermissionClass {
    Unique,
    Mutable,
    Shared,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CheriProjectionRecord {
    pub function: String,
    pub path: String,
    pub root_size: u64,
    pub offset: Option<u64>,
    pub referent_size: u64,
    pub permission_class: CheriPermissionClass,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CodeGenerationOptions {
    pub mangle_symbols: bool,
    pub export_global_symbols: bool,
    pub pointer_address_space: u16,
}

impl<'ctx> CodeGeneration<'ctx> {
    pub fn pointer_address_space(&self) -> AddressSpace {
        AddressSpace::from(self.options.pointer_address_space)
    }

    pub fn cheri_projection_records(&self) -> Vec<CheriProjectionRecord> {
        self.cheri_projection_records.borrow().clone()
    }
}
