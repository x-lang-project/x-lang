use super::*;

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
