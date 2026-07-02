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

mod float_intrinsics;
mod integer_intrinsics;
pub mod intrinsics;
mod invariants;
mod located_error;
mod metadata;
mod statements;
mod substitute;
mod type_names;
mod type_symbols;
mod types;
mod validate;

pub use float_intrinsics::{FloatIntrinsic, FloatOperation, FloatWidth};
pub use integer_intrinsics::{
    BitIntrinsic, BitOperation, BitWidth, OverflowIntrinsic, OverflowOperation, OverflowType,
    RotateDirection,
};
pub use intrinsics::ControlIntrinsic;
pub use intrinsics::IntrinsicScope;
pub use intrinsics::IntrinsicSignature;
pub use intrinsics::IntrinsicVisibility;
pub use intrinsics::LayoutIntrinsic;
pub use intrinsics::MemoryIntrinsic;
pub use invariants::InvariantBinding;
pub use located_error::{LocatedError, SourceError};
pub use metadata::{
    Attribute, ConstValue, DeclarationKey, GenericArgument, GenericParameter, Lifetime, SourceSpan,
    Visibility,
};
pub use statements::ExternParameter;
pub use statements::Parameter;
pub use statements::ParameterRole;
pub use substitute::substitute_self_in_type;
pub use type_names::{primitive_type_from_name, type_from_name};
pub use type_symbols::symbol_name_component;
pub use types::{Layout, Operator, ReferenceQualifiers, Type, UnaryOperator};
pub use validate::MAX_AST_EXPRESSION_DEPTH;
pub use validate::MAX_AST_NODES;
pub use validate::MAX_AST_STATEMENT_DEPTH;

pub const ENTRY_POINT_NAME: &str = "main";
