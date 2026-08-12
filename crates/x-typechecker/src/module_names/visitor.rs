use super::*;

impl TypeVisitorMut for ModuleQualifier<'_> {
    fn visit_type(&mut self, value_type: &mut Type) {
        match value_type {
            Type::Array(inner)
            | Type::FixedSizedArray(inner, _)
            | Type::RawPointer { inner, .. }
            | Type::Reference { inner, .. }
            | Type::Simd { element: inner, .. } => self.visit_type(inner),
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
            ExpressionKind::Match { scrutinee, arms } => {
                self.visit_type(&mut expression.value_type);
                self.visit_expression(scrutinee);
                for arm in arms {
                    self.value_body(&mut arm.body, arm.result.as_mut(), Some(&arm.pattern));
                }

                return;
            }
            ExpressionKind::FunctionCall { target, .. } => {
                if target.path.segments.len() == 1 && !self.value_is_local(target.leaf_name()) {
                    let leaf = target.leaf_name().to_owned();

                    if self.functions.contains(&leaf) {
                        target.path.segments.insert(
                            0,
                            x_ast::CallSegment {
                                name: self.prefix.to_owned(),
                                type_arguments: Vec::new(),
                            },
                        );
                    }
                }
            }
            ExpressionKind::Identifier(name) if !self.value_is_local(name) => {
                self.qualify(name, self.variables)
            }
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
