mod parameters;
use pest::iterators::Pair;

use crate::attributes::{self, FunctionAttribute};
use crate::block;
use crate::{
    Rule,
    expression::parse_expression,
    lowering_error::{LoweringError, LoweringResult},
    types::parse_type,
};
use parameters::*;
use x_ast::{Expression, ExpressionKind, GenericParameter, Parameter, Statement, Type};
use x_diagnostics::parser_messages::ExpectedSyntax;
use x_diagnostics::parser_messages::ParserIssue;

pub fn parse_function_definition(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let mut inner = pair.into_inner();

    let mut attributes = Vec::new();

    while let Some(current_pair) = inner.peek() {
        if current_pair.as_rule() == Rule::attribute {
            let attribute = inner.next().ok_or_else(|| {
                LoweringError::missing(&error_pair, ExpectedSyntax::FunctionAttribute)
            })?;
            let name = attributes::function_attribute(attribute.as_str())
                .map_err(|detail| LoweringError::invalid(&attribute, detail))?;
            attributes.push(name);
        } else {
            break;
        }
    }

    let mut is_memoised = attributes.contains(&FunctionAttribute::Memoise);
    let mut is_pure = false;
    let mut is_static = false;
    let mut is_multi = false;
    let mut is_throws = false;
    let is_exported = attributes.contains(&FunctionAttribute::NoMangle);

    while let Some(current_pair) = inner.peek() {
        if current_pair.as_rule() == Rule::modifiers {
            let member = inner.next().ok_or_else(|| {
                LoweringError::missing(&error_pair, ExpectedSyntax::FunctionModifier)
            })?;

            match member.as_str().trim() {
                "memoise" => is_memoised = true,
                "pure" => is_pure = true,
                "static" => is_static = true,
                "multi" => is_multi = true,
                text if text == "throws" || text.starts_with("throws(") => is_throws = true,

                _ => {}
            }
        } else {
            break;
        }
    }

    if inner.peek().is_some_and(|current_pair| {
        matches!(current_pair.as_rule(), Rule::string | Rule::char_literal)
    }) {
        inner.next();
    }

    let name = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::FunctionName))?
        .as_str()
        .to_string();
    let generic_parameters = if inner.peek().map(|current_pair| current_pair.as_rule())
        == Some(Rule::generic_parameter_definition_list)
    {
        let generic_pair = inner.next().ok_or_else(|| {
            LoweringError::missing(&error_pair, ExpectedSyntax::GenericParameters)
        })?;

        parse_generic_parameters(generic_pair)?
    } else {
        Vec::new()
    };

    let parameters_pair = inner.next().ok_or_else(|| {
        LoweringError::missing(&error_pair, ExpectedSyntax::FunctionParameterList)
    })?;
    let mut parameters = parse_function_parameters(parameters_pair)?;

    if is_static {
        for parameter in parameters.iter_mut() {
            parameter.is_static = true;
        }
    }

    let mut return_type = Type::Void;

    if inner.peek().map(|current_pair| current_pair.as_rule()) == Some(Rule::return_type_annotation)
    {
        let return_pair = inner
            .next()
            .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::ReturnType))?;
        return_type = parse_type(return_pair)?;
    }

    let body = if inner.peek().map(|current_pair| current_pair.as_rule()) == Some(Rule::block) {
        let body_pair = inner
            .next()
            .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::FunctionBody))?;

        Some(Box::new(block::parse_block_statements(body_pair)?))
    } else {
        None
    };

    Ok(Statement::Function {
        name,
        generic_parameters,
        parameters,
        return_type,
        body,
        is_pure,
        is_memoised,
        is_static,
        is_multi,
        is_throws,
        is_exported,
    })
}

fn parse_generic_parameters(pair: Pair<Rule>) -> LoweringResult<Vec<GenericParameter>> {
    crate::statement::generics::parse_generic_parameters(
        pair,
        crate::statement::generics::LifetimeSpelling::Bare,
    )
}

pub(crate) fn parse_arguments(pair: Option<Pair<Rule>>) -> LoweringResult<Vec<Expression>> {
    match pair {
        Some(pair) => pair.into_inner().map(parse_expression).collect(),
        None => Ok(Vec::new()),
    }
}

pub fn parse_function_call(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = crate::source_span_from_pair(&pair);
    let error_pair = pair.clone();
    let mut inner = pair.into_inner().peekable();
    let name_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::CallableName))?;
    let mut target = crate::call_targets::parse_target(name_pair)?;

    if let Some(next_pair) = inner.peek()
        && next_pair.as_rule() == Rule::generic_arguments
    {
        crate::call_targets::attach_leaf_arguments(&mut target, next_pair.clone())?;
        inner.next();
    }

    let arguments = parse_arguments(inner.find(|pair| pair.as_rule() == Rule::arguments))?;

    Ok(Expression {
        span: Some(pair_span),
        value_type: Type::Unknown,
        kind: ExpressionKind::FunctionCall { target, arguments },
    })
}
