mod provenance;

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
    fn local_binding(&self, name: &str) -> Option<((usize, String), Type)> {
        let function_scope = self.function_contexts.last()?.scope_depth;
        let (resolved_name, value_type, is_global) = self
            .symbols
            .resolve_variable_with_scope(name, self.current_function_name().map(String::as_str))?;

        if is_global {
            return None;
        }

        let (depth, _) = self
            .symbols
            .variables
            .iter()
            .enumerate()
            .rev()
            .find(|(_, scope)| scope.contains_key(&resolved_name))?;

        (depth >= function_scope).then(|| ((depth, resolved_name), self.resolve(&value_type)))
    }

    // Pointee ownership requires provenance beyond the local binding.
    pub(crate) fn is_local_owned_assignment(&self, target: &Expression) -> bool {
        match &target.kind {
            ExpressionKind::Identifier(name) => self.local_binding(name).is_some(),
            ExpressionKind::FieldAccess { object, .. } => {
                !self.is_indirect_reference_slot(target, object)
                    && self.is_local_projection_base(object)
            }
            ExpressionKind::ArrayAccess { array, .. } => {
                !self.is_indirect_reference_slot(target, array)
                    && self.is_local_projection_base(array)
            }
            ExpressionKind::Dereference { expression } => {
                // Reference-slot writes can invalidate another binding's provenance.
                !matches!(
                    self.resolve(&target.value_type),
                    Type::Reference { .. } | Type::RawPointer { .. }
                ) && self.is_local_reference_value(expression)
            }
            _ => false,
        }
    }

    fn is_indirect_reference_slot(&self, target: &Expression, base: &Expression) -> bool {
        matches!(
            self.resolve(&target.value_type),
            Type::Reference { .. } | Type::RawPointer { .. }
        ) && matches!(
            self.resolve(&base.value_type),
            Type::Reference { .. } | Type::RawPointer { .. }
        )
    }

    fn is_local_projection_base(&self, base: &Expression) -> bool {
        match self.resolve(&base.value_type) {
            Type::Reference { .. } => self.is_local_reference_value(base),
            // Unsized arrays are handles; fixed arrays own their inline storage.
            Type::RawPointer { .. } | Type::Array(_) => false,
            _ => self.is_local_owned_assignment(base),
        }
    }

    pub(crate) fn is_local_reference_value(&self, value: &Expression) -> bool {
        match &value.kind {
            ExpressionKind::AddressOf {
                is_mut: true,
                expression,
                ..
            } => {
                self.is_local_owned_assignment(expression)
                    && !matches!(
                        self.resolve(&expression.value_type),
                        Type::Reference { .. } | Type::RawPointer { .. }
                    )
            }
            ExpressionKind::Identifier(_)
            | ExpressionKind::FieldAccess { .. }
            | ExpressionKind::ArrayAccess { .. } => {
                self.local_reference_paths(value).contains(&Vec::new())
            }
            _ => false,
        }
    }

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
