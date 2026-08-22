use super::*;

pub(super) fn ensure_static_substitution_is_unambiguous(
    statements: &[Statement],
    substitutions: &HashMap<String, Expression>,
) -> Result<(), TypeError> {
    use x_ast::ExpressionVisitor;

    struct ShadowCheck<'a> {
        substitutions: &'a HashMap<String, Expression>,
        error: Option<TypeError>,
    }

    impl ExpressionVisitor for ShadowCheck<'_> {
        fn visit_statement(&mut self, statement: &Statement) {
            if self.error.is_some() {
                return;
            }

            let message = match statement {
                Statement::VariableDeclaration { pattern, .. }
                    if pattern_names(pattern)
                        .iter()
                        .any(|name| self.substitutions.contains_key(name)) =>
                {
                    Some(FrontendMessage::StaticParameterShadowedByLocal)
                }
                Statement::Constant { name, .. } if self.substitutions.contains_key(name) => {
                    Some(FrontendMessage::StaticParameterShadowedByConstant)
                }
                Statement::ForRangeLoop { variable, .. }
                | Statement::ForEachLoop { variable, .. }
                    if self.substitutions.contains_key(variable) =>
                {
                    Some(FrontendMessage::StaticParameterShadowedByLoop)
                }
                Statement::Function { .. } => {
                    Some(FrontendMessage::StaticParameterCapturedByNestedFunction)
                }
                _ => None,
            };

            if let Some(message) = message {
                self.error = Some(TypeError::inference(message));
                return;
            }

            x_ast::walk_statement_expressions(self, statement);
        }

        fn visit_expression(&mut self, expression: &Expression) {
            if self.error.is_some()
                || matches!(expression.kind, ExpressionKind::AnonymousFunction { .. })
            {
                return;
            }

            if let ExpressionKind::Match { arms, .. } = &expression.kind
                && arms.iter().any(|arm| {
                    pattern_names(&arm.pattern)
                        .iter()
                        .any(|name| self.substitutions.contains_key(name))
                })
            {
                self.error = Some(
                    TypeError::inference(FrontendMessage::StaticParameterShadowedByMatch)
                        .at_source(expression.span),
                );
                return;
            }

            x_ast::walk_expression_children(self, expression);
        }
    }

    if substitutions.is_empty() {
        return Ok(());
    }

    let mut check = ShadowCheck {
        substitutions,
        error: None,
    };

    check.visit_statements(statements);

    match check.error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

pub(super) fn static_literal_key(expression: &Expression) -> Option<StaticValue> {
    StaticValue::from_expression(expression)
}

pub(super) fn substitute_statements(
    statements: &mut [Statement],
    substitutions: &HashMap<String, Expression>,
) -> Result<(), TypeError> {
    for statement in statements {
        if let Statement::VariableDeclaration { pattern, .. } = statement
            && pattern_names(pattern)
                .iter()
                .any(|name| substitutions.contains_key(name))
        {
            return Err(TypeError::inference(
                FrontendMessage::StaticParameterShadowedByLocal,
            ));
        }

        substitute_statement(statement, substitutions)?;
    }

    Ok(())
}

pub(super) fn pattern_names(pattern: &Pattern) -> &[String] {
    match pattern {
        Pattern::Identifier(name) => slice::from_ref(name),
        Pattern::Tuple(names)
        | Pattern::Enum {
            bindings: names, ..
        } => names,
        Pattern::Wildcard => &[],
    }
}

pub(super) fn substitute_statement(
    statement: &mut Statement,
    substitutions: &HashMap<String, Expression>,
) -> Result<(), TypeError> {
    let mut substitute =
        |expression: &mut Expression| substitute_expression(expression, substitutions);

    visit_statement_expressions_mut(statement, DeclarationBodies::Skip, &mut substitute)
}

pub(super) fn substitute_expression(
    expression: &mut Expression,
    substitutions: &HashMap<String, Expression>,
) -> Result<(), TypeError> {
    if let ExpressionKind::Identifier(name) = &expression.kind
        && let Some(value) = substitutions.get(name)
    {
        *expression = value.clone();

        return Ok(());
    }

    walk_expression_children_mut(expression, &mut |child| {
        substitute_expression(child, substitutions)
    })
}
