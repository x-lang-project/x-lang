use super::*;

pub(super) fn apply_array_postfix(base: Type, postfix: Pair<Rule>) -> LoweringResult<Type> {
    let raw = postfix.as_str().trim();

    if raw == "[]" {
        return Ok(Type::Array(Box::new(base)));
    }

    let error_pair = postfix.clone();
    let length = postfix
        .into_inner()
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::ArrayLength))?;
    let current_length = parse_array_length(length.as_str(), &error_pair)?;

    Ok(Type::FixedSizedArray(Box::new(base), current_length))
}

pub(super) fn parse_array_length(raw: &str, error_pair: &Pair<Rule>) -> LoweringResult<usize> {
    raw.replace('_', "").parse::<usize>().map_err(|error| {
        LoweringError::invalid(
            error_pair,
            ParserIssue::InvalidArrayLength(error.to_string()),
        )
    })
}

pub(super) fn parse_function_type(pair: Pair<Rule>) -> LoweringResult<Type> {
    let mut parameters: Vec<Type> = Vec::new();
    let mut return_value: Type = Type::Void;

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::r#type => {
                parameters.push(parse_type(child)?);
            }
            Rule::type_specifier => {
                if let Some(inner_type) = child.into_inner().next() {
                    return_value = parse_type(inner_type)?;
                }
            }
            _ => {}
        }
    }

    Ok(Type::Function {
        parameters,
        return_value: Box::new(return_value),
    })
}

pub(super) fn parse_array_type(pair: Pair<Rule>) -> LoweringResult<Type> {
    let error_pair = pair.clone();
    let mut inner = pair.into_inner();
    let Some(inner_type) = inner.next() else {
        return Err(LoweringError::missing(
            &error_pair,
            ExpectedSyntax::ArrayElementType,
        ));
    };

    let base = parse_type(inner_type)?;

    Ok(match inner.next() {
        Some(length) => {
            let current_length = parse_array_length(length.as_str(), &length)?;

            Type::FixedSizedArray(Box::new(base), current_length)
        }
        None => Type::Array(Box::new(base)),
    })
}

pub(super) fn parse_tuple_type(pair: Pair<Rule>) -> LoweringResult<Type> {
    let elements: Vec<Type> = pair
        .into_inner()
        .map(parse_type)
        .collect::<LoweringResult<_>>()?;

    Ok(Type::Tuple(elements))
}

pub(super) fn parse_reference_type(pair: Pair<Rule>) -> LoweringResult<Type> {
    let error_pair = pair.clone();
    let mut inner = pair.into_inner().peekable();

    let syntax = crate::reference_syntax::ReferenceSyntax::consume(&mut inner);

    let Some(base_pair) = inner.next() else {
        return Err(LoweringError::missing(
            &error_pair,
            ExpectedSyntax::ReferencedType,
        ));
    };

    let inner_type = parse_type(base_pair)?;

    Ok(syntax.into_type(inner_type))
}

pub(super) fn parse_pointer_type(pair: Pair<Rule>) -> LoweringResult<Type> {
    let error_pair = pair.clone();
    let mut inner = pair.into_inner().peekable();
    let mut is_mut = false;

    if inner.peek().map(|current_pair| current_pair.as_rule()) == Some(Rule::kw_mut) {
        is_mut = true;
        inner.next();
    } else if inner.peek().map(|current_pair| current_pair.as_rule()) == Some(Rule::kw_const) {
        inner.next();
    }

    let Some(base_pair) = inner.next() else {
        return Err(LoweringError::missing(
            &error_pair,
            ExpectedSyntax::PointerTargetType,
        ));
    };

    let base_type = parse_type(base_pair)?;

    Ok(Type::RawPointer {
        is_mut,
        inner: Box::new(base_type),
    })
}
