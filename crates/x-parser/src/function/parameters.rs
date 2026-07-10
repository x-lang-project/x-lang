use super::*;

pub(super) fn parse_function_parameters(
    parameters_pair: Pair<Rule>,
) -> LoweringResult<Vec<Parameter>> {
    let mut parameters = Vec::new();

    if parameters_pair.as_rule() != Rule::function_parameter_list {
        return Err(LoweringError::invalid(
            &parameters_pair,
            ParserIssue::ExpectedFunctionParameterList,
        ));
    }

    if let Some(list) = parameters_pair.into_inner().next() {
        for pair in list.into_inner() {
            let parameter_error = pair.clone();
            let parameter_item = pair.into_inner().next().ok_or_else(|| {
                LoweringError::missing(&parameter_error, ExpectedSyntax::Parameter)
            })?;
            let item_error = parameter_item.clone();

            let parameter = match parameter_item.as_rule() {
                Rule::parameter_normal => parse_normal_parameter(parameter_item)?,
                Rule::parameter_self => parse_self_parameter(parameter_item),
                Rule::parameter_owned_self => parse_owned_receiver(parameter_item),
                _ => {
                    return Err(LoweringError::invalid(
                        &item_error,
                        ParserIssue::UnsupportedFunctionParameterShape,
                    ));
                }
            };

            if parameter.is_receiver() && !parameters.is_empty() {
                return Err(LoweringError::invalid(
                    &item_error,
                    ParserIssue::UnsupportedFunctionParameterShape,
                ));
            }

            parameters.push(parameter);
        }
    }

    Ok(parameters)
}

pub(super) fn parse_self_parameter(pair: Pair<Rule>) -> Parameter {
    let mut inner = pair.into_inner().peekable();
    let syntax = crate::reference_syntax::ReferenceSyntax::consume(&mut inner);
    let is_mut = syntax.is_mut;

    Parameter {
        role: x_ast::ParameterRole::Receiver,
        name: "self".to_string(),
        value_type: syntax.into_type(Type::TypeParameter("Self".to_string())),
        is_static: false,
        is_mut,
    }
}

pub(super) fn parse_normal_parameter(pair: Pair<Rule>) -> LoweringResult<Parameter> {
    let error_pair = pair.clone();
    let mut pin = pair.into_inner().peekable();

    let mut explicit_static = false;

    if pin
        .peek()
        .is_some_and(|value| value.as_rule() == Rule::kw_static)
    {
        explicit_static = true;
        pin.next();
    }

    let explicit_unique = if pin
        .peek()
        .is_some_and(|value| value.as_rule() == Rule::unique_kw)
    {
        pin.next();

        true
    } else {
        false
    };

    let is_mut = if pin
        .peek()
        .is_some_and(|value| value.as_rule() == Rule::kw_mut)
    {
        pin.next();

        true
    } else {
        false
    };

    let name = pin
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::ParameterName))?
        .as_str()
        .to_string();
    let type_rule = pin.next().ok_or_else(|| {
        LoweringError::invalid(&error_pair, ParserIssue::UnsupportedFunctionParameterShape)
    })?;
    let mut value_type = parse_type(type_rule)?;
    let is_static = explicit_static;

    if explicit_unique && let Type::Reference { is_unique, .. } = &mut value_type {
        *is_unique = true;
    }

    Ok(Parameter {
        role: x_ast::ParameterRole::Argument,
        name,
        value_type,
        is_static,
        is_mut,
    })
}

fn parse_owned_receiver(pair: Pair<Rule>) -> Parameter {
    let mut is_static = false;
    let mut is_mut = false;

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::kw_static => is_static = true,
            Rule::kw_mut => is_mut = true,
            _ => {}
        }
    }

    Parameter {
        role: x_ast::ParameterRole::Receiver,
        name: "self".into(),
        value_type: Type::TypeParameter("Self".into()),
        is_static,
        is_mut,
    }
}
