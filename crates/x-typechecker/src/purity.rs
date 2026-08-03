use std::collections::HashSet;

use crate::FunctionSignature;
use x_ast::intrinsics::EffectCategory;
use x_ast::{Expression, ExpressionKind, Type};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum LocalProjection {
    Field(String),
    TupleIndex(usize),
    Index(i64),
    AnyIndex,
}
