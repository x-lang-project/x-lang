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

mod metadata;

mod substitute;
mod type_names;
mod type_symbols;
mod types;
mod validate;

pub use metadata::Lifetime;

pub use substitute::substitute_self_in_type;
pub use type_names::{primitive_type_from_name, type_from_name};
pub use type_symbols::symbol_name_component;
pub use types::{Layout, Operator, ReferenceQualifiers, Type, UnaryOperator};
pub use validate::MAX_AST_EXPRESSION_DEPTH;
pub use validate::MAX_AST_NODES;
pub use validate::MAX_AST_STATEMENT_DEPTH;
