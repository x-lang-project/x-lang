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
pub mod expression_lowerer;
pub mod lower;

pub use call_targets::{CallSite, ResolvedCall};
pub use contracts::{
    CallContract, DEFAULT_MEMOISE_CAPACITY, MemoiseCallContract, MemoiseIndexing,
    MemoiseKeyEncoding, MemoiseSynchronization,
};
pub use ir::{
    BasicBlock, BlockIdentifier, LocalDeclaration, LocalIdentifier, MirStatement, MirTerminator,
    Operand, Place, Rvalue,
};
pub use lower::{FunctionInput, MirLowerer, lower_program};
pub use obligations::{
    ContractObligationError, ContractObligations, FunctionObligations, LoweredMemoryEffects,
    MutationObligation, TailTransferObligation,
};
pub use program::{FunctionMeta, GlobalItem, MirBody, MirProgram};
pub use validate::MirValidationError;
