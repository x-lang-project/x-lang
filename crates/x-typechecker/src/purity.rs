use std::collections::HashSet;

use crate::FunctionSignature;
use crate::TypeChecker;
use x_ast::intrinsics::EffectCategory;
use x_ast::{Expression, ExpressionKind, Type};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum LocalProjection {
    Field(String),
    TupleIndex(usize),
    Index(i64),
    AnyIndex,
}

pub(crate) enum ImpureOperation<'a> {
    Panic,
    ParallelLoop,
    Assignment,
    InlineAssembly,
    GlobalRead(&'a str),
    BuiltinCall {
        name: &'a str,
        effect: EffectCategory,
    },
}

impl ImpureOperation<'_> {
    fn effect(&self) -> EffectCategory {
        match self {
            Self::Panic => EffectCategory::Panic,
            Self::ParallelLoop => EffectCategory::Parallel,
            Self::Assignment | Self::GlobalRead(_) | Self::InlineAssembly => EffectCategory::Unsafe,
            Self::BuiltinCall { effect, .. } => *effect,
        }
    }
}
