use pest::iterators::Pair;

use crate::expression;
use crate::types;
use crate::{
    Rule,
    lowering_error::{LoweringError, LoweringResult},
};
use x_ast::Expression;
use x_ast::Type;
use x_ast::{Pattern, Statement};
use x_diagnostics::parser_messages::ExpectedSyntax;

pub fn parse_const_declaration(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let (name, type_annotation, value) = parse_declaration_fields(pair, "constant")?;

    Ok(Statement::Constant {
        name,
        type_annotation,
        value,
    })
}

pub fn parse_static_declaration(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let (name, type_annotation, value) = parse_declaration_fields(pair, "static")?;

    Ok(Statement::VariableDeclaration {
        pattern: Pattern::Identifier(name),
        type_annotation,
        value,
    })
}

fn parse_declaration_fields(
    pair: Pair<Rule>,
    kind: &str,
) -> LoweringResult<(String, Option<Type>, Expression)> {
    let error_pair = pair.clone();
    let mut name = None;
    let mut type_annotation = None;
    let mut value = None;

    for next in pair.into_inner() {
        match next.as_rule() {
            Rule::identifier => name = Some(next.as_str().to_string()),
            Rule::r#type | Rule::type_specifier => {
                type_annotation = Some(types::parse_type(next)?);
            }
            Rule::expression => value = Some(expression::parse_expression(next)?),
            _ => {}
        }
    }

    let name = name.ok_or_else(|| {
        LoweringError::missing(&error_pair, ExpectedSyntax::BindingNameFor(kind.to_owned()))
    })?;
    let value = value.ok_or_else(|| {
        LoweringError::missing(
            &error_pair,
            ExpectedSyntax::BindingValueFor(kind.to_owned()),
        )
    })?;

    Ok((name, type_annotation, value))
}
