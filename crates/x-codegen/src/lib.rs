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

pub use codegen::CheriPermissionClass;
pub use codegen::CheriProjectionRecord;
pub use codegen::CodeGeneration;
pub use codegen::CodeGenerationOptions;
pub use codegen::ObjectEmissionOptions;
pub use codegen::OptimizationPolicy;
pub use codegen::create_target_machine;
pub use codegen::create_target_machine_for_initialized_target;
pub use codegen::initialize_target_for_triple;
pub use codegen::llvm_optimization_level;
pub use codegen::write_ir_to_object;
pub use codegen::write_module_to_object;
pub use codegen::write_module_to_object_with_target_machine;
pub use errors::CodeGenerationError;
pub use symbols::{GlobalSymbol, MangleConfig, SymbolKind, SymbolTable, collect_symbol_table};
