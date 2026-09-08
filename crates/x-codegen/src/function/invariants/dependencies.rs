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

struct Dependencies<'a> {
    receiver: &'a x_ast::InvariantBinding,
    paths: Vec<Vec<String>>,
    unknown: bool,
}

impl ExpressionVisitor for Dependencies<'_> {
    fn visit_expression(&mut self, expression: &Expression) {
        if matches!(expression.kind, ExpressionKind::FieldAccess { .. })
            && let Some(fields) = path(expression, self.receiver)
        {
            self.paths.push(fields);
            return;
        }

        match &expression.kind {
            _ if self.receiver.refers_to(expression) => self.unknown = true,
            ExpressionKind::FieldAccess { object, .. } if object.value_type.pointee().is_some() => {
                self.unknown = true;
            }
            ExpressionKind::FunctionCall { .. }
            | ExpressionKind::MethodCall { .. }
            | ExpressionKind::Dereference { .. }
            | ExpressionKind::AddressOf { .. }
            | ExpressionKind::ArrayAccess { .. }
            | ExpressionKind::AnonymousFunction { .. } => {
                self.unknown = true;
            }
            _ => {}
        }

        x_ast::walk_expression_children(self, expression);
    }
}

fn overlaps(invariant: &StructInvariant, mutation: &[String]) -> bool {
    let mut dependencies = Dependencies {
        receiver: &invariant.receiver,
        paths: Vec::new(),
        unknown: false,
    };
    dependencies.visit_expression(&invariant.predicate);

    dependencies.unknown
        || dependencies.paths.iter().any(|dependency| {
            dependency
                .iter()
                .zip(mutation)
                .all(|(first_value, second_value)| first_value == second_value)
        })
}
