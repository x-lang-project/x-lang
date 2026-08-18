use super::*;
use x_diagnostics::frontend_messages::FrontendMessage;

// Large cast literals use the AST's i64 payload before storage normalisation.
fn cast_literal_source_type(expression: &Expression, inferred: Type) -> Type {
    if inferred != Type::I32 {
        return inferred;
    }

    let value = match &expression.kind {
        ExpressionKind::Integer(value) => Some(i128::from(*value)),
        ExpressionKind::UnaryOperator {
            operator: UnaryOperator::Negate,
            expression,
        } => match expression.kind {
            ExpressionKind::Integer(value) => Some(-i128::from(value)),
            _ => None,
        },
        _ => None,
    };

    if value.is_some_and(|value| value < i128::from(i32::MIN) || value > i128::from(i32::MAX)) {
        Type::I64
    } else {
        inferred
    }
}

fn unresolved_integer_literal(expression: &Expression) -> bool {
    if expression.value_type != Type::Unknown {
        return false;
    }

    match &expression.kind {
        ExpressionKind::Integer(_) => true,
        ExpressionKind::UnaryOperator {
            operator: UnaryOperator::Negate,
            expression: inner,
        } => inner.value_type == Type::Unknown && matches!(inner.kind, ExpressionKind::Integer(_)),
        _ => false,
    }
}
