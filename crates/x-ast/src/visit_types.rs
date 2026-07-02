use crate::{Expression, ExpressionKind, Statement, Type};

// Hooks receive whole types; declarations retain their own scopes.
pub trait TypeVisitorMut {
    fn visit_type(&mut self, value_type: &mut Type);

    fn visit_statements(&mut self, statements: &mut [Statement]) {
        for statement in statements {
            self.visit_statement(statement);
        }
    }

    fn visit_expressions(&mut self, expressions: &mut [Expression]) {
        for expression in expressions {
            self.visit_expression(expression);
        }
    }

    fn visit_statement(&mut self, statement: &mut Statement) {
        walk_statement_types_mut(self, statement);
    }

    fn visit_expression(&mut self, expression: &mut Expression) {
        walk_expression_types_mut(self, expression);
    }

    fn visit_call_target(&mut self, target: &mut crate::CallTarget) {
        if let Some(crate::CallResolution::Builtin { type_arguments, .. }) = &mut target.resolution
        {
            for argument in type_arguments {
                self.visit_type(argument);
            }
        }

        for segment in &mut target.path.segments {
            for argument in &mut segment.type_arguments {
                self.visit_type(argument);
            }
        }
    }
}

pub fn walk_statement_types_mut<V: TypeVisitorMut + ?Sized>(
    visitor: &mut V,
    statement: &mut Statement,
) {
    walk_statement_children!(visitor, statement, {
        Statement::Function { parameters, return_type, body, .. } => {
            for param in parameters {
                visitor.visit_type(&mut param.value_type);
            }
            visitor.visit_type(return_type);
            if let Some(body) = body {
                visitor.visit_statements(body);
            }
        },
        Statement::VariableDeclaration { type_annotation, value, .. }
        | Statement::Constant { type_annotation, value, .. } => {
            if let Some(value_type) = type_annotation {
                visitor.visit_type(value_type);
            }
            visitor.visit_expression(value);
        },
        Statement::TypeAlias { value_type, .. } => visitor.visit_type(value_type),
    });
}

pub fn walk_expression_types_mut<V: TypeVisitorMut + ?Sized>(
    visitor: &mut V,
    expression: &mut Expression,
) {
    visitor.visit_type(&mut expression.value_type);
    if let ExpressionKind::FunctionCall { target, .. } | ExpressionKind::MethodCall { target, .. } =
        &mut expression.kind
    {
        visitor.visit_call_target(target);
    }

    walk_expression_children!(visitor, &mut expression.kind, iter_mut, as_mut_slice, as_mut, {
        ExpressionKind::Cast { expression, to } => {
            visitor.visit_expression(expression);
            visitor.visit_type(to);
        },
        ExpressionKind::TypeLiteral(value_type) | ExpressionKind::SizeOf(value_type) => visitor.visit_type(value_type),
        ExpressionKind::AnonymousFunction { body, return_type, .. } => {
            if let Some(value_type) = return_type {
                visitor.visit_type(value_type);
            }
            visitor.visit_statements(body);
        },
    });
}
