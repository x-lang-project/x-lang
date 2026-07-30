use std::collections::HashSet;

use super::Invariants;
use crate::{MirBody, Place};
use x_ast::{
    Expression, ExpressionKind, ExpressionVisitor, InvariantBinding, StructInvariant, Type,
};

pub(super) fn needs_context(place: &Place, body: &MirBody, invariants: &Invariants) -> bool {
    let mut projection = place;
    let mut path = Vec::new();

    loop {
        match projection {
            Place::Field {
                base, field_name, ..
            } => {
                path.insert(0, Some(field_name.clone()));
                let value_type = body.place_type(base);
                let mut value_type = &value_type;

                while let Type::Reference { inner, .. } | Type::RawPointer { inner, .. } =
                    value_type
                {
                    value_type = inner;
                }

                if let Type::Custom(name) | Type::GenericInstance { name, .. } = value_type
                    && let Some(invariant) = invariants.get(name)
                    && overlaps(invariant, &path)
                {
                    return true;
                }

                projection = base;
            }

            _ => return false,
        }
    }
}

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

impl ExpressionVisitor for Dependencies<'_> {
    fn visit_expression(&mut self, expression: &Expression) {
        if matches!(expression.kind, ExpressionKind::FieldAccess { .. })
            && let Some(path) = field_path(expression, self.receiver)
        {
            self.fields.insert(path);
            return;
        }

        match &expression.kind {
            _ if self.receiver.refers_to(expression) => self.unknown = true,
            ExpressionKind::FunctionCall { .. }
            | ExpressionKind::MethodCall { .. }
            | ExpressionKind::Dereference { .. }
            | ExpressionKind::AddressOf { .. }
            | ExpressionKind::AnonymousFunction { .. }
            | ExpressionKind::ArrayAccess { .. } => self.unknown = true,
            _ => {}
        }

        x_ast::walk_expression_children(self, expression);
    }
}

fn overlaps(invariant: &StructInvariant, path: &[Option<String>]) -> bool {
    let mut dependencies = Dependencies {
        receiver: &invariant.receiver,
        fields: HashSet::new(),
        unknown: false,
    };
    dependencies.visit_expression(&invariant.predicate);

    dependencies.unknown
        || dependencies.fields.iter().any(|dependency| {
            dependency
                .iter()
                .zip(path)
                .all(|(field, component)| component.as_ref() == Some(field))
        })
}
