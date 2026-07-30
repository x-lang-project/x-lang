use std::collections::HashSet;

use super::Invariants;
use crate::{MirBody, Place};
use x_ast::{
    Expression, ExpressionKind, ExpressionVisitor, InvariantBinding, StructInvariant, Type,
};

fn field_path(expression: &Expression, receiver: &InvariantBinding) -> Option<Vec<String>> {
    match &expression.kind {
        _ if receiver.refers_to(expression) => Some(Vec::new()),
        ExpressionKind::FieldAccess { object, field } => {
            let mut path = field_path(object, receiver)?;
            path.push(field.clone());

            Some(path)
        }
        _ => None,
    }
}

struct Dependencies<'a> {
    receiver: &'a InvariantBinding,
    fields: HashSet<Vec<String>>,
    unknown: bool,
}
