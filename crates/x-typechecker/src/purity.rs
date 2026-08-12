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

impl LocalProjection {
    fn covers(&self, projection: &Self) -> bool {
        self == projection
            || matches!(
                (self, projection),
                (Self::AnyIndex, Self::Index(_) | Self::AnyIndex)
            )
    }

    fn invalidated_by(&self, changed: &Self) -> bool {
        matches!(
            (self, changed),
            (Self::AnyIndex, Self::Index(_)) | (Self::Index(_), Self::AnyIndex)
        )
    }
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
    fn description(&self) -> String {
        match self {
            Self::Panic => "panic statement".to_string(),
            Self::ParallelLoop => "parallel for loop".to_string(),
            Self::Assignment => "assignment in pure function".to_string(),
            Self::InlineAssembly => "inline assembly".to_string(),
            Self::GlobalRead(name) => format!("global read `{name}`"),
            Self::BuiltinCall { name, .. } => (*name).to_string(),
        }
    }

    fn effect(&self) -> EffectCategory {
        match self {
            Self::Panic => EffectCategory::Panic,
            Self::ParallelLoop => EffectCategory::Parallel,
            Self::Assignment | Self::GlobalRead(_) | Self::InlineAssembly => EffectCategory::Unsafe,
            Self::BuiltinCall { effect, .. } => *effect,
        }
    }
}

impl TypeChecker {
    pub(crate) fn leave_purity_scope(&mut self) {
        let closing = self.symbols.variables.len().saturating_sub(1);
        self.pure_local_references
            .retain(|(depth, _), _| *depth < closing);
    }

    pub(crate) fn current_function_is_pure(&self) -> bool {
        self.function_contexts
            .last()
            .is_some_and(|context| context.is_pure)
    }
}
