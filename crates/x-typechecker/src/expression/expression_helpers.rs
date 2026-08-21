use x_ast::{Expression, ExpressionKind, Type, UnaryOperator};

pub(crate) fn is_integer_literal(expression: &Expression) -> bool {
    matches!(expression.kind, ExpressionKind::Integer(_))
        || matches!(
            &expression.kind,
            ExpressionKind::UnaryOperator { operator: UnaryOperator::Negate, expression }
                if matches!(expression.kind, ExpressionKind::Integer(_))
        )
}

pub(super) fn is_method_generic_placeholder(value_type: &Type) -> bool {
    matches!(value_type, Type::TypeParameter(_))
}
