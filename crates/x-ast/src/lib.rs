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

pub mod intrinsics;

mod validate;

pub use validate::MAX_AST_EXPRESSION_DEPTH;
pub use validate::MAX_AST_NODES;
pub use validate::MAX_AST_STATEMENT_DEPTH;
