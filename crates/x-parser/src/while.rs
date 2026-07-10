use pest::iterators::Pair;

use crate::{
    Rule,
    block::parse_block_statements,
    expression::parse_expression,
    lowering_error::{LoweringError, LoweringResult, required_child},
};
use x_ast::{Expression, ExpressionKind, Statement, Type};
use x_diagnostics::parser_messages::ExpectedSyntax;

pub fn parse_while_loop(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let mut inner = pair.into_inner();
    let condition_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::WhileCondition))?;
    let body_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::WhileBody))?;
    let condition = parse_expression(condition_pair)?;
    let body = parse_block_statements(body_pair)?;

    Ok(Statement::WhileLoop {
        condition: Box::new(condition),
        body,
    })
}

pub(crate) fn parse_infinite_loop(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let body = required_child(pair, ExpectedSyntax::LoopBody)?;
    let body = parse_block_statements(body)?;

    Ok(Statement::WhileLoop {
        condition: Box::new(Expression {
            span: None,
            value_type: Type::Boolean,
            kind: ExpressionKind::Boolean(true),
        }),
        body,
    })
}
