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
mod invariant_references;
mod ir;
mod local_declarations;
mod obligations;
mod program;
mod source_names;
mod type_queries;
mod validate;

pub mod errors;

pub use call_targets::{CallSite, ResolvedCall};
pub use contracts::CallContract;
pub use contracts::MemoiseCallContract;
pub use contracts::MemoiseIndexing;
pub use contracts::MemoiseKeyEncoding;
pub use contracts::MemoiseSynchronization;
pub use ir::{
    BasicBlock, BlockIdentifier, LocalDeclaration, LocalIdentifier, MirStatement, MirTerminator,
    Operand, Place, Rvalue,
};

pub use obligations::{
    ContractObligationError, ContractObligations, FunctionObligations, LoweredMemoryEffects,
    MutationObligation, TailTransferObligation,
};
pub use program::{FunctionMeta, GlobalItem, MirBody, MirProgram};
pub use validate::MirValidationError;
