use crate::{Expression, ExpressionKind, Statement};

// Declaration bodies require an explicit visitor hook.
pub trait ExpressionVisitor {
    fn visit_statements(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.visit_statement(statement);
        }
    }

    fn visit_expressions(&mut self, expressions: &[Expression]) {
        for expression in expressions {
            self.visit_expression(expression);
        }
    }

    fn visit_statement(&mut self, statement: &Statement) {
        walk_statement_expressions(self, statement);
    }

    fn visit_expression(&mut self, expression: &Expression) {
        walk_expression_children(self, expression);
    }
}

pub fn walk_statement_expressions<V: ExpressionVisitor + ?Sized>(
    visitor: &mut V,
    statement: &Statement,
) {
    walk_statement_children!(visitor, statement, {
        Statement::Function { .. } => {},
        Statement::VariableDeclaration { value, .. }
        | Statement::Constant { value, .. } => visitor.visit_expression(value),
        Statement::TypeAlias { .. } => {},
    });
}

pub fn walk_expression_children<V: ExpressionVisitor + ?Sized>(
    visitor: &mut V,
    expression: &Expression,
) {
    walk_expression_children!(visitor, &expression.kind, iter, as_slice, as_ref, {
        ExpressionKind::Cast { expression, .. } => visitor.visit_expression(expression),
        ExpressionKind::TypeLiteral(_) | ExpressionKind::SizeOf(_) => {},
        ExpressionKind::AnonymousFunction { body, .. } => visitor.visit_statements(body),
    });
}
