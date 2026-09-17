#![cfg_attr(
    not(test),
    deny(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unreachable,
        clippy::unwrap_used
    )
)]

pub mod codegen;
pub mod errors;
pub mod function;
mod function_helpers;
pub mod operators;
pub mod symbols;
#[cfg(windows)]
mod target_initializers;

pub use codegen::{
    CheriPermissionClass, CheriProjectionRecord, CodeGeneration, CodeGenerationOptions,
    FunctionDeclaration, ObjectEmissionOptions, OptimizationPolicy, create_target_machine,
    create_target_machine_for_initialized_target, initialize_target_for_triple,
    llvm_optimization_level, write_ir_to_object, write_module_to_object,
    write_module_to_object_with_target_machine,
};
pub use errors::CodeGenerationError;
pub use symbols::{GlobalSymbol, MangleConfig, SymbolKind, SymbolTable, collect_symbol_table};
