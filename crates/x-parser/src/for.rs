use std::iter::Peekable;

use pest::iterators::{Pair, Pairs};

use crate::{
    Rule,
    block::parse_block_statements,
    expression::parse_expression,
    lowering_error::{LoweringError, LoweringResult},
};
use x_ast::Statement;
use x_diagnostics::parser_messages::ExpectedSyntax;

fn parse_loop_binding(
    inner: &mut Peekable<Pairs<'_, Rule>>,
    origin: &Pair<Rule>,
    expected: ExpectedSyntax,
) -> LoweringResult<(String, bool)> {
    let is_parallel = inner
        .peek()
        .is_some_and(|pair| pair.as_rule() == Rule::parallel_kw);

    if is_parallel {
        inner.next();
    }

    let variable = inner
        .next()
        .ok_or_else(|| LoweringError::missing(origin, expected))?
        .as_str()
        .to_string();

    Ok((variable, is_parallel))
}

pub fn parse_for_range_loop(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let mut inner = pair.into_inner().peekable();

    let (variable, is_parallel) =
        parse_loop_binding(&mut inner, &error_pair, ExpectedSyntax::RangeVariable)?;
    let start_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::RangeStart))?;
    let end_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::RangeEnd))?;
    let body_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::RangeBody))?;
    let start = Box::new(parse_expression(start_pair)?);
    let end = Box::new(parse_expression(end_pair)?);
    let body = parse_block_statements(body_pair)?;

    Ok(Statement::ForRangeLoop {
        variable,
        start,
        end,
        body,
        is_parallel,
    })
}

pub fn parse_for_each_loop(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let mut inner = pair.into_inner().peekable();

    let (variable, is_parallel) =
        parse_loop_binding(&mut inner, &error_pair, ExpectedSyntax::IterationVariable)?;
    let iterator_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::IteratorExpression))?;
    let body_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::IterationBody))?;
    let iterator = Box::new(parse_expression(iterator_pair)?);
    let body = parse_block_statements(body_pair)?;

    Ok(Statement::ForEachLoop {
        variable,
        iterator,
        body,
        is_parallel,
    })
}
