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

mod ir;
mod local_declarations;
mod obligations;

mod validate;

pub use ir::{
    BasicBlock, BlockIdentifier, LocalDeclaration, LocalIdentifier, MirStatement, MirTerminator,
    Operand, Place, Rvalue,
};

pub use obligations::ContractObligationError;

pub use validate::MirValidationError;
