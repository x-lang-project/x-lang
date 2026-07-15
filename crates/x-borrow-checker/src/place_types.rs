use super::{BorrowChecker, parameter_origins};
use x_ast::{Expression, ExpressionKind, Type};

impl BorrowChecker {
    pub(super) fn is_safe_reference_value(&self, expression: &Expression) -> bool {
        matches!(self.place_type(expression), Type::Reference { .. })
    }

    pub(super) fn place_type(&self, expression: &Expression) -> Type {
        match &expression.kind {
            ExpressionKind::Identifier(name) => self
                .lookup_variable(name)
                .map(|(binding, _)| binding.value_type.clone())
                .unwrap_or_else(|| expression.value_type.clone()),
            ExpressionKind::Dereference {
                expression: pointer,
            } => match self.place_type(pointer) {
                Type::Reference { inner, .. } | Type::RawPointer { inner, .. } => *inner,
                _ => expression.value_type.clone(),
            },
            ExpressionKind::FieldAccess { object, field } => {
                let mut value_type = self.place_type(object);

                while let Type::Reference { inner, .. } = value_type {
                    value_type = *inner;
                }

                parameter_origins::fields(&value_type, &self.context.struct_layouts)
                    .and_then(|fields| {
                        fields
                            .into_iter()
                            .find(|(name, _)| name == field)
                            .map(|(_, value_type)| value_type)
                    })
                    .unwrap_or_else(|| expression.value_type.clone())
            }
            ExpressionKind::ArrayAccess { array, .. } => {
                let mut value_type = self.place_type(array);

                while let Type::Reference { inner, .. } = value_type {
                    value_type = *inner;
                }

                match value_type {
                    Type::Array(inner)
                    | Type::FixedSizedArray(inner, _)
                    | Type::RawPointer { inner, .. } => *inner,
                    _ => expression.value_type.clone(),
                }
            }
            _ => expression.value_type.clone(),
        }
    }
}
