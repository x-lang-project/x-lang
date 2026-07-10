use pest::iterators::Pair;

use crate::{
    Rule,
    r#if::parse_if_expression,
    lowering_error::{LoweringError, LoweringResult},
    source_span_from_pair,
    term::parse_term,
    types::parse_type,
};
use matching::parse_match_expression;
use x_ast::{Expression, ExpressionKind, Statement, Type};
use x_diagnostics::parser_messages::{ExpectedSyntax, ParserIssue};

mod binary;
mod matching;
mod unary;

pub use unary::parse_unary;

pub fn parse_expression(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(&pair);
    let expression = match pair.as_rule() {
        Rule::expression => {
            let error_pair = pair.clone();
            let inner = pair.into_inner().next().ok_or_else(|| {
                LoweringError::missing(&error_pair, ExpectedSyntax::InnerExpression)
            })?;

            parse_expression(inner)?
        }
        Rule::ternary_expression => parse_ternary_expression(pair)?,
        Rule::match_expression => parse_match_expression(pair)?,
        _ => binary::parse_binary(pair.into_inner())?,
    };

    Ok(expression.with_span(pair_span))
}

fn parse_primary(pair: Pair<Rule>) -> LoweringResult<Expression> {
    match pair.as_rule() {
        Rule::expression | Rule::ternary_expression => parse_expression(pair),
        Rule::postfix_expression
        | Rule::postfix
        | Rule::cast_expression
        | Rule::value_identifier
        | Rule::identifier => parse_term(pair),

        Rule::if_statement | Rule::if_expression => parse_if_expression(pair),
        Rule::match_expression => parse_match_expression(pair),

        Rule::custom_type | Rule::basic_type | Rule::reference_type | Rule::type_identifier => {
            let value_type = parse_type(pair)?;

            Ok(Expression {
                span: None,
                value_type: Type::Unknown,
                kind: ExpressionKind::TypeLiteral(value_type),
            })
        }
        Rule::unary_expression => parse_unary(pair),
        Rule::binary_expression => parse_expression(pair),
        _ => Err(LoweringError::invalid(
            &pair,
            ParserIssue::UnsupportedExpressionRule,
        )),
    }
}

fn parse_ternary_expression(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(&pair);
    let error_pair = pair.clone();
    let mut inner = pair.into_inner();
    let condition_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::TernaryCondition))?;
    let condition = parse_expression(condition_pair)?;

    let Some(then_pair) = inner.next() else {
        return Ok(condition.with_span(pair_span));
    };

    let else_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::TernaryElseArm))?;
    let then_expression = parse_expression(then_pair)?;
    let else_expression = parse_expression(else_pair)?;

    Ok(Expression {
        span: Some(pair_span),
        value_type: Type::Unknown,
        kind: ExpressionKind::If {
            condition: Box::new(condition),
            then_block: vec![Statement::Return {
                value: Some(then_expression),
            }],
            else_block: Some(vec![Statement::Return {
                value: Some(else_expression),
            }]),
        },
    })
}
