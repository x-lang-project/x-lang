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
mod display;
mod expressions;
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
#[macro_use]
mod walk_children;
mod visit_expression;
mod visit_types;

pub use call_targets::{CallPath, CallResolution, CallSegment, CallTarget};
pub use expressions::{Expression, ExpressionKind, MatchArm, Pattern, StringLiteral, StringPart};
pub use float_intrinsics::{FloatIntrinsic, FloatOperation, FloatWidth};
pub use integer_intrinsics::{
    BitIntrinsic, BitOperation, BitWidth, OverflowIntrinsic, OverflowOperation, OverflowType,
    RotateDirection,
};
pub use intrinsics::ControlIntrinsic;
pub use intrinsics::Intrinsic;
pub use intrinsics::IntrinsicDefinition;
pub use intrinsics::IntrinsicScope;
pub use intrinsics::IntrinsicSignature;
pub use intrinsics::IntrinsicVisibility;
pub use intrinsics::LayoutIntrinsic;
pub use intrinsics::MemoryIntrinsic;
pub use invariants::{InvariantBinding, StructInvariant};
pub use located_error::{LocatedError, SourceError};
pub use metadata::{
    Attribute, ConstValue, DeclarationKey, GenericArgument, GenericParameter, Lifetime, SourceSpan,
    Visibility,
};
pub use statements::{
    ExternParameter, ImplDefinition, Parameter, ParameterRole, Program, Statement,
    StructDefinition, StructInitialisation, TraitDefinition,
};
pub use substitute::{
    substitute_self_in_expression, substitute_self_in_statement, substitute_self_in_type,
};
pub use type_names::{primitive_type_from_name, type_from_name};
pub use type_symbols::{symbol_name_component, type_symbol_component, type_symbol_components};
pub use types::{Layout, Operator, ReferenceQualifiers, Type, UnaryOperator};
pub use validate::MAX_AST_EXPRESSION_DEPTH;
pub use validate::MAX_AST_NODES;
pub use validate::MAX_AST_STATEMENT_DEPTH;
pub use visit_expression::{
    ExpressionVisitor, walk_expression_children, walk_statement_expressions,
};
pub use visit_types::{TypeVisitorMut, walk_expression_types_mut, walk_statement_types_mut};

pub const ENTRY_POINT_NAME: &str = "main";
