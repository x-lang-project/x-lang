use pest::iterators::Pair;

use crate::{
    Rule,
    block::{parse_block_statements, parse_value_block},
    expression::parse_expression,
    lowering_error::{LoweringError, LoweringResult},
    source_span_from_pair,
};
use x_ast::{Expression, ExpressionKind, Statement, Type};
use x_diagnostics::parser_messages::ExpectedSyntax;
use x_diagnostics::parser_messages::ParserIssue;

pub fn parse_if_statement(pair: Pair<Rule>) -> LoweringResult<Statement> {
    parse_if(pair, false)
}

fn parse_if(pair: Pair<Rule>, produces_value: bool) -> LoweringResult<Statement> {
    let parse_body = if produces_value {
        parse_value_block
    } else {
        parse_block_statements
    };

    let error_pair = pair.clone();
    let mut inner = pair.into_inner();

    let condition_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::IfCondition))?;
    let condition = Box::new(parse_expression(condition_pair)?);

    let then_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::IfBody))?;
    let then_block = parse_body(then_pair)?;

    let else_block = match inner.next() {
        None => None,
        Some(otherwise) => Some(match otherwise.as_rule() {
            Rule::block => parse_body(otherwise)?,
            Rule::if_statement if produces_value => vec![Statement::Expression {
                expression: parse_if_expression(otherwise)?,
            }],
            Rule::if_statement => vec![parse_if_statement(otherwise)?],
            _ => {
                return Err(LoweringError::invalid(
                    &otherwise,
                    ParserIssue::InvalidElseBody,
                ));
            }
        }),
    };

    Ok(Statement::If {
        condition,
        then_block,
        else_block,
    })
}

pub fn parse_if_expression(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(&pair);
    let error_pair = pair.clone();

    if pair.as_rule() == Rule::if_expression {
        let inner = pair
            .into_inner()
            .next()
            .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::InnerIfStatement))?;

        return Ok(parse_if_expression(inner)?.with_span(pair_span));
    }

    match parse_if(pair, true)? {
        Statement::If {
            condition,
            then_block,
            else_block,
        } => Ok(Expression {
            span: Some(pair_span),
            value_type: Type::Unknown,
            kind: ExpressionKind::If {
                condition,
                then_block,
                else_block,
            },
        }),
        _ => Err(LoweringError::invalid(
            &error_pair,
            ParserIssue::InvalidIfLowering,
        )),
    }
}
