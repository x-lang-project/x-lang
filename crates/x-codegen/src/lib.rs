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

pub mod symbols;
#[cfg(windows)]
mod target_initializers;

pub use codegen::CheriPermissionClass;
pub use codegen::CheriProjectionRecord;
pub use codegen::CodeGeneration;
pub use codegen::CodeGenerationOptions;
pub use codegen::create_target_machine;
pub use codegen::create_target_machine_for_initialized_target;
pub use codegen::initialize_target_for_triple;
pub use codegen::llvm_optimization_level;
pub use errors::CodeGenerationError;
pub use symbols::GlobalSymbol;
pub use symbols::MangleConfig;
pub use symbols::SymbolKind;
pub use symbols::SymbolTable;
