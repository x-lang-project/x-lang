mod containers;
use pest::iterators::Pair;

use crate::{
    Rule,
    lowering_error::{LoweringError, LoweringResult},
};
use containers::*;
use x_ast::Type;
use x_diagnostics::parser_messages::ExpectedSyntax;
use x_diagnostics::parser_messages::ParserIssue;

pub fn parse_type(pair: Pair<Rule>) -> LoweringResult<Type> {
    let error_pair = pair.clone();
    let value_type = match pair.as_rule() {
        Rule::type_specifier
        | Rule::return_type_annotation
        | Rule::type_atom
        | Rule::trait_path => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::InnerType))?;

            return parse_type(inner);
        }

        Rule::function_type => parse_function_type(pair)?,

        Rule::r#type => parse_type_with_postfixes(pair)?,

        Rule::array_type => parse_array_type(pair)?,

        Rule::basic_type => Type::from_primitive_name(pair.as_str()).ok_or_else(|| {
            let detail = match pair.as_str() {
                "i128" | "u128" => ParserIssue::UnsupportedWideInteger,
                other => ParserIssue::UnknownPrimitiveType(other.to_owned()),
            };

            LoweringError::invalid(&error_pair, detail)
        })?,

        Rule::identifier | Rule::value_identifier | Rule::type_identifier => {
            Type::Custom(pair.as_str().to_string())
        }

        Rule::custom_type => parse_named_type(pair)?,

        Rule::tuple_type => parse_tuple_type(pair)?,

        Rule::reference_type => parse_reference_type(pair)?,

        Rule::pointer_type => parse_pointer_type(pair)?,

        _ => {
            return Err(LoweringError::invalid(
                &error_pair,
                ParserIssue::UnsupportedTypeRule,
            ));
        }
    };

    Ok(value_type)
}

fn parse_type_with_postfixes(pair: Pair<Rule>) -> LoweringResult<Type> {
    let error_pair = pair.clone();
    let mut inner = pair.into_inner();
    let Some(first) = inner.next() else {
        return Err(LoweringError::missing(
            &error_pair,
            ExpectedSyntax::BaseType,
        ));
    };

    let mut base = parse_type(first)?;

    for current_pair in inner {
        if current_pair.as_rule() == Rule::array_postfix {
            base = apply_array_postfix(base, current_pair)?;
        }
    }

    Ok(base)
}

fn parse_named_type(pair: Pair<Rule>) -> LoweringResult<Type> {
    let error_pair = pair.clone();
    let raw = pair.as_str().trim().to_string();
    let name = raw
        .split_once('<')
        .map(|(name, _)| name.trim().to_string())
        .unwrap_or_else(|| raw.to_string());
    let arguments = pair
        .into_inner()
        .filter(|current_pair| current_pair.as_rule() == Rule::generic_argument)
        .collect::<Vec<_>>();

    if name == "simd" {
        return parse_simd_type(&arguments, &error_pair);
    }

    let mut type_arguments = Vec::new();

    for generic in arguments {
        let mut argument_inner = generic.clone().into_inner();
        let argument = argument_inner
            .next()
            .ok_or_else(|| LoweringError::missing(&generic, ExpectedSyntax::GenericArgument))?;

        match argument.as_rule() {
            Rule::type_specifier | Rule::r#type => type_arguments.push(parse_type(argument)?),
            _ => {
                return Err(LoweringError::invalid(
                    &argument,
                    ParserIssue::UnsupportedGenericTypeArgument,
                ));
            }
        }
    }

    Ok(if type_arguments.is_empty() {
        Type::Custom(name)
    } else {
        Type::GenericInstance {
            name,
            type_arguments,
        }
    })
}

fn parse_simd_type(arguments: &[Pair<Rule>], error_pair: &Pair<Rule>) -> LoweringResult<Type> {
    if let [element_argument, lanes_argument] = arguments {
        let element_pair = element_argument
            .clone()
            .into_inner()
            .find(|pair| matches!(pair.as_rule(), Rule::type_specifier | Rule::r#type))
            .ok_or_else(|| LoweringError::missing(error_pair, ExpectedSyntax::SimdElementType))?;
        let element = parse_type(element_pair)?;
        let lanes = lanes_argument
            .clone()
            .into_inner()
            .find(|pair| pair.as_rule() == Rule::integer)
            .and_then(|pair| pair.as_str().replace('_', "").parse::<usize>().ok());

        if let Some(lanes) = lanes {
            return Ok(Type::Simd {
                element: Box::new(element),
                lanes,
            });
        }
    }

    Err(LoweringError::invalid(
        error_pair,
        ParserIssue::InvalidSimdArguments,
    ))
}

pub(crate) fn parse_generic_bound_list(pair: Pair<Rule>) -> LoweringResult<Vec<Type>> {
    let mut bounds = Vec::new();

    for bound in pair.into_inner() {
        if let Some(bound) = parse_generic_bound(bound)? {
            bounds.push(bound);
        }
    }

    Ok(bounds)
}

fn parse_generic_bound(pair: Pair<Rule>) -> LoweringResult<Option<Type>> {
    Ok(match pair.as_rule() {
        Rule::generic_bound => pair
            .into_inner()
            .next()
            .map(parse_generic_bound)
            .transpose()?
            .flatten(),
        Rule::type_specifier | Rule::r#type | Rule::custom_type | Rule::basic_type => {
            Some(parse_type(pair)?)
        }
        Rule::callable_bound => Some(Type::Custom(pair.as_str().trim().to_string())),
        Rule::relaxed_bound => pair
            .into_inner()
            .find(|child| matches!(child.as_rule(), Rule::type_specifier | Rule::r#type))
            .map(parse_type)
            .transpose()?,
        Rule::lifetime => Some(Type::Custom(pair.as_str().to_string())),
        _ => None,
    })
}
