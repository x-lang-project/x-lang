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

mod call_targets;
mod contracts;

mod ir;
mod local_declarations;
mod obligations;

mod validate;

pub use call_targets::ResolvedCall;
pub use contracts::CallContract;
pub use contracts::MemoiseCallContract;
pub use contracts::MemoiseIndexing;
pub use contracts::MemoiseKeyEncoding;
pub use contracts::MemoiseSynchronization;
pub use ir::{
    BasicBlock, BlockIdentifier, LocalDeclaration, LocalIdentifier, MirStatement, MirTerminator,
    Operand, Place, Rvalue,
};

pub use obligations::ContractObligationError;

pub use validate::MirValidationError;
