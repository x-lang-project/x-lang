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

impl FunctionCompiler<'_, '_> {
    pub(super) fn invariant_depends_on_write(&self, owner: &Place, destination: &Place) -> bool {
        let mut current = destination;
        let mut mutation = Vec::new();

        while current != owner {
            match current {
                Place::Field {
                    base, field_name, ..
                } => {
                    if base.as_ref() != owner && self.get_place_type(base).pointee().is_some() {
                        return true;
                    }

                    mutation.insert(0, field_name.clone());
                    current = base;
                }
                _ => return true,
            }
        }

        let mut value_type = self.get_place_type(owner);

        while let Type::Reference { inner, .. } | Type::RawPointer { inner, .. } = value_type {
            value_type = *inner;
        }

        let name = match value_type {
            Type::Custom(name) | Type::GenericInstance { name, .. } => name,
            _ => return false,
        };

        let invariant = self
            .codegen
            .resolve_struct_name(&name)
            .and_then(|canonical| self.codegen.records.get(canonical))
            .or_else(|| self.codegen.records.get(&name))
            .and_then(|record| record.invariant());

        invariant.is_some_and(|invariant| overlaps(invariant, &mutation))
    }
}
