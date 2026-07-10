use super::*;

pub(super) fn parse_variable_declaration(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let mut inner = pair.into_inner().peekable();

    while inner.peek().is_some_and(|current_pair| {
        matches!(
            current_pair.as_rule(),
            Rule::kw_mut | Rule::unique_kw | Rule::variable_modifier
        )
    }) {
        inner.next();
    }

    let target_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::BindingPattern))?;
    let pattern = match target_pair.as_rule() {
        Rule::let_pattern | Rule::tuple_literal => {
            let names: Vec<String> = target_pair
                .into_inner()
                .filter(|current_pair| current_pair.as_rule() == Rule::identifier)
                .map(|current_pair| current_pair.as_str().to_string())
                .collect();

            match names.as_slice() {
                [name] => Pattern::Identifier(name.clone()),
                _ => Pattern::Tuple(names),
            }
        }
        _ => {
            return Err(LoweringError::invalid(
                &target_pair,
                ParserIssue::UnsupportedVariableBindingPattern,
            ));
        }
    };

    let mut type_annotation: Option<Type> = None;
    let mut value: Option<Expression> = None;

    if let Some(next) = inner.peek() {
        match next.as_rule() {
            Rule::r#type => {
                let type_pair = inner.next().ok_or_else(|| {
                    LoweringError::missing(&error_pair, ExpectedSyntax::BindingType)
                })?;
                type_annotation = Some(parse_type(type_pair)?);

                if let Some(value_pair) = inner.next() {
                    value = Some(parse_expression(value_pair)?);
                }
            }
            Rule::expression => {
                let value_pair = inner.next().ok_or_else(|| {
                    LoweringError::missing(&error_pair, ExpectedSyntax::BindingValue)
                })?;
                value = Some(parse_expression(value_pair)?);
            }
            _ => {}
        }
    }

    let value = value.ok_or_else(|| {
        LoweringError::invalid(&error_pair, ParserIssue::UnsupportedUninitializedBinding)
    })?;

    Ok(Statement::VariableDeclaration {
        pattern,
        type_annotation,
        value,
    })
}

pub(super) fn parse_assignment_statement(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let mut inner = pair.into_inner();

    let lval_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::AssignmentTarget))?;
    let _assign = inner.next();
    let right_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::AssignmentValue))?;

    let target = parse_l_value(lval_pair)?;
    let value = parse_expression(right_pair)?;

    Ok(Statement::Expression {
        expression: Expression {
            span: None,
            value_type: Type::Void,
            kind: ExpressionKind::Assignment {
                target: Box::new(target),
                value: Box::new(value),
            },
        },
    })
}

pub(super) fn parse_l_value(pair: Pair<Rule>) -> LoweringResult<Expression> {
    if pair.as_rule() != Rule::l_value {
        return Err(LoweringError::invalid(&pair, ParserIssue::ExpectedLvalue));
    }

    let error_pair = pair.clone();
    let first = pair
        .into_inner()
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::AssignableTarget))?;

    match first.as_rule() {
        Rule::primary_lvalue => {
            let inner_node = first.into_inner().next().ok_or_else(|| {
                LoweringError::missing(&error_pair, ExpectedSyntax::PrimaryLvalue)
            })?;

            if inner_node.as_rule() == Rule::l_value {
                parse_l_value(inner_node)
            } else if inner_node.as_rule() == Rule::postfix_expression {
                parse_term(inner_node)
            } else {
                Err(LoweringError::invalid(
                    &inner_node,
                    ParserIssue::InvalidPrimaryLvalue,
                ))
            }
        }
        Rule::deref | Rule::address_of => parse_term(first),
        _ => Err(LoweringError::invalid(
            &first,
            ParserIssue::InvalidLvalueShape,
        )),
    }
}
