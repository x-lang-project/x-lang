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

pub use codegen::CheriPermissionClass;
pub use codegen::CheriProjectionRecord;
pub use codegen::CodeGeneration;
pub use codegen::CodeGenerationOptions;
pub use errors::CodeGenerationError;

#[cfg(windows)]
mod target_initializers;
