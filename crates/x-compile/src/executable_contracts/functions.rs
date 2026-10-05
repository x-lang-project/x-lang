use super::*;

fn invariant_capture(expression: &Expression, iteration_variable: &str) -> bool {
    struct Dependencies<'a> {
        iteration_variable: &'a str,
        reads_iteration: bool,
    }

    impl x_ast::ExpressionVisitor for Dependencies<'_> {
        fn visit_expression(&mut self, expression: &Expression) {
            if matches!(&expression.kind, ExpressionKind::Identifier(name)
                if name == self.iteration_variable)
            {
                self.reads_iteration = true;
            }

            if !self.reads_iteration {
                x_ast::walk_expression_children(self, expression);
            }
        }
    }

    let mut dependencies = Dependencies {
        iteration_variable,
        reads_iteration: false,
    };
    x_ast::ExpressionVisitor::visit_expression(&mut dependencies, expression);

    !dependencies.reads_iteration
}

pub(super) fn executable_invariant(invariant: &x_ast::StructInvariant) -> bool {
    executable_invariant_expression(&invariant.predicate, &invariant.receiver)
}

fn executable_invariant_expression(
    expression: &Expression,
    receiver: &x_ast::InvariantBinding,
) -> bool {
    match &expression.kind {
        ExpressionKind::Integer(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Boolean(_)
        | ExpressionKind::Character(_) => true,
        ExpressionKind::Identifier(_) => receiver.refers_to(expression),
        ExpressionKind::FieldAccess { object, .. }
        | ExpressionKind::UnaryOperator {
            expression: object, ..
        } => executable_invariant_expression(object, receiver),
        ExpressionKind::BinaryOperator { left, right, .. } => {
            executable_invariant_expression(left, receiver)
                && executable_invariant_expression(right, receiver)
        }
        ExpressionKind::Cast { expression, to } => {
            expression.value_type == *to && executable_invariant_expression(expression, receiver)
        }
        _ => false,
    }
}
