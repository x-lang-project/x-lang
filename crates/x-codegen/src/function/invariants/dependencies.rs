use super::*;
use x_ast::{ExpressionVisitor, StructInvariant};

fn path(expression: &Expression, receiver: &x_ast::InvariantBinding) -> Option<Vec<String>> {
    match &expression.kind {
        _ if receiver.refers_to(expression) => Some(Vec::new()),
        ExpressionKind::FieldAccess { object, field } if object.value_type.pointee().is_none() => {
            let mut result = path(object, receiver)?;
            result.push(field.clone());

            Some(result)
        }
        _ => None,
    }
}
