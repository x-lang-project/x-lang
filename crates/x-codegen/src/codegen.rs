mod attributes;
mod functions;
mod globals;
mod items;
pub use items::FunctionDeclaration;
mod jit;
mod obligations;
mod optimize;
pub use optimize::{
    ObjectEmissionOptions, OptimizationPolicy, create_target_machine,
    create_target_machine_for_initialized_target, initialize_target_for_triple,
    llvm_optimization_level, write_ir_to_object, write_module_to_object,
    write_module_to_object_with_target_machine,
};
mod records;
mod runtime;
mod structs;
mod symbols;
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

pub(crate) const CHERI_CAPABILITY_ADDRESS_SPACE: u16 = 200;

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

impl CheriPermissionClass {
    pub const fn from_qualifiers(qualifiers: x_ast::ReferenceQualifiers) -> Self {
        if qualifiers.is_unique {
            Self::Unique
        } else if qualifiers.is_mut {
            Self::Mutable
        } else {
            Self::Shared
        }
    }

    pub const fn trace_name(self) -> &'static str {
        match self {
            Self::Unique => "unique",
            Self::Mutable => "mutable",
            Self::Shared => "shared",
        }
    }
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

impl CheriProjectionRecord {
    pub fn to_tsv(&self) -> String {
        let offset = self
            .offset
            .map(|value| value.to_string())
            .unwrap_or_else(|| "dynamic".to_string());

        format!(
            "x-cheri-v1\t{}\t{}\t{}\t{}\t{}\t{}",
            self.function,
            self.path,
            self.root_size,
            offset,
            self.referent_size,
            self.permission_class.trace_name()
        )
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CodeGenerationOptions {
    pub mangle_symbols: bool,
    pub export_global_symbols: bool,
    pub pointer_address_space: u16,
}

impl CodeGenerationOptions {
    pub const fn cheri_purecap() -> Self {
        Self {
            mangle_symbols: false,
            export_global_symbols: false,
            pointer_address_space: CHERI_CAPABILITY_ADDRESS_SPACE,
        }
    }
}

impl<'ctx> CodeGeneration<'ctx> {
    pub fn new(
        context: &'ctx Context,
        module_name: &str,
        optimisation_level: OptimizationLevel,
    ) -> Result<Self, CodeGenerationError> {
        Self::with_options(
            context,
            module_name,
            optimisation_level,
            CodeGenerationOptions::default(),
        )
    }

    pub fn with_options(
        context: &'ctx Context,
        module_name: &str,
        optimisation_level: OptimizationLevel,
        options: CodeGenerationOptions,
    ) -> Result<Self, CodeGenerationError> {
        let host_triple = targets::TargetMachine::get_default_triple();
        let host_machine = create_target_machine(&host_triple, 0)?;

        Ok(Self::with_target_machine(
            context,
            module_name,
            optimisation_level,
            options,
            &host_machine,
        ))
    }

    pub fn with_target_machine(
        context: &'ctx Context,
        module_name: &str,
        optimisation_level: OptimizationLevel,
        options: CodeGenerationOptions,
        machine: &targets::TargetMachine,
    ) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        module.set_data_layout(&machine.get_target_data().get_data_layout());
        info!("Created LLVM module {module_name}");

        CodeGeneration {
            context,
            builder,
            module,
            execution_engine: RefCell::new(None),
            jit_optimization_level: optimisation_level,
            records: records::RecordCatalogue::default(),
            options,
            function_symbols: HashMap::new(),
            global_symbols: HashMap::new(),
            cheri_projection_records: RefCell::new(Vec::new()),
            contract_obligations: RefCell::new(x_mir::ContractObligations::default()),
        }
    }

    pub fn generate(&mut self, mir_program: &MirProgram) -> Result<(), CodeGenerationError> {
        mir_program
            .validate()
            .map_err(|error| CodeGenerationError::InvalidMir(error.to_string()))?;
        *self.contract_obligations.borrow_mut() = mir_program
            .contract_obligations()
            .map_err(|error| CodeGenerationError::InvalidMir(error.to_string()))?;
        self.prepare_symbol_tables(mir_program)?;
        self.register_stdlib_stubs();
        self.declare_struct_types(mir_program)?;
        self.declare_static_storage_items(mir_program)?;
        self.declare_function_signatures(mir_program)?;
        self.compile_function_bodies(mir_program)?;
        self.ensure_optional_main();
        self.discharge_lowered_effect_obligations();
        self.module
            .verify()
            .map_err(|error| CodeGenerationError::LlvmVerificationFailure(error.to_string()))?;

        Ok(())
    }

    pub fn pointer_address_space(&self) -> AddressSpace {
        AddressSpace::from(self.options.pointer_address_space)
    }

    pub fn record_cheri_projection(&self, record: CheriProjectionRecord) {
        self.cheri_projection_records.borrow_mut().push(record);
    }

    pub fn cheri_projection_records(&self) -> Vec<CheriProjectionRecord> {
        self.cheri_projection_records.borrow().clone()
    }
}

fn is_exported_entry_symbol(name: &str) -> bool {
    matches!(name, x_ast::ENTRY_POINT_NAME | "kmain" | "_start")
}
