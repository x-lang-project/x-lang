use std::collections::HashSet;

use crate::TypeChecker;

impl TypeChecker {
    pub(crate) fn register_struct_invariant(
        &mut self,
        name: String,
        invariant: &x_ast::StructInvariant,
    ) {
        use x_ast::{ExpressionKind, ExpressionVisitor};

        struct Dependencies<'a> {
            receiver: &'a x_ast::InvariantBinding,
            fields: HashSet<Vec<String>>,
            unknown: bool,
        }
        fn field_path(
            expression: &x_ast::Expression,
            receiver: &x_ast::InvariantBinding,
        ) -> Option<Vec<String>> {
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
        impl ExpressionVisitor for Dependencies<'_> {
            fn visit_expression(&mut self, expression: &x_ast::Expression) {
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

        let mut dependencies = Dependencies {
            receiver: &invariant.receiver,
            fields: HashSet::new(),
            unknown: false,
        };
        dependencies.visit_expression(&invariant.predicate);
        let fields = (!dependencies.unknown).then_some(dependencies.fields);
        self.symbols.invariant_structs.insert(name.clone());
        self.symbols
            .invariant_dependencies
            .entry(name)
            .and_modify(|existing| match (existing.as_mut(), fields.as_ref()) {
                (Some(existing), Some(fields)) => existing.extend(fields.iter().cloned()),
                _ => *existing = None,
            })
            .or_insert(fields);
    }
}
