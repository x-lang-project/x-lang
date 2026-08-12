use super::*;

impl TypeVisitorMut for ModuleQualifier<'_> {
    fn visit_type(&mut self, value_type: &mut Type) {
        match value_type {
            Type::Tuple(items) => {
                for value_type in items {
                    self.visit_type(value_type);
                }
            }
            Type::Function {
                parameters,
                return_value,
            } => {
                for value_type in parameters {
                    self.visit_type(value_type);
                }

                self.visit_type(return_value);
            }
            _ => {}
        }
    }

    fn visit_statement(&mut self, statement: &mut Statement) {
        match statement {
            _ => x_ast::walk_statement_types_mut(self, statement),
        }
    }

    fn visit_expression(&mut self, expression: &mut Expression) {
        match &mut expression.kind {
            ExpressionKind::StructInstantiate(initialise)
                if !self.type_is_local(&initialise.name) =>
            {
                self.qualify(&mut initialise.name, self.types)
            }
            _ => {}
        }

        x_ast::walk_expression_types_mut(self, expression);
    }
}
