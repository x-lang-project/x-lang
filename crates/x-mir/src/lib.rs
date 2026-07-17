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

pub use ir::LocalDeclaration;
pub use ir::LocalIdentifier;
pub use ir::MirStatement;
pub use ir::Operand;
pub use ir::Place;
pub use ir::Rvalue;

pub use obligations::ContractObligationError;
