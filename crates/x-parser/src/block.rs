use pest::iterators::Pair;

use crate::{
    Rule,
    expression::parse_expression,
    lowering_error::{LoweringError, LoweringResult},
    statement::parse_statement,
};
use x_ast::Expression;
use x_ast::Statement;
use x_diagnostics::parser_messages::ExpectedSyntax;
use x_diagnostics::parser_messages::ParserIssue;

pub fn parse_block(pair: Pair<Rule>) -> LoweringResult<Statement> {
    Ok(Statement::Block {
        statements: parse_block_statements(pair)?,
    })
}

pub(crate) fn parse_block_statements(pair: Pair<Rule>) -> LoweringResult<Vec<Statement>> {
    let (mut statements, result) = parse_expression_block(pair)?;

    if let Some(expression) = result {
        statements.push(Statement::Return {
            value: Some(*expression),
        });
    }

    Ok(statements)
}

pub(crate) fn parse_value_block(pair: Pair<Rule>) -> LoweringResult<Vec<Statement>> {
    let (mut statements, result) = parse_expression_block(pair)?;

    if let Some(expression) = result {
        statements.push(Statement::Expression {
            expression: *expression,
        });
    }

    Ok(statements)
}

type ExpressionBlock = (Vec<Statement>, Option<Box<Expression>>);

pub(crate) fn parse_expression_block(pair: Pair<Rule>) -> LoweringResult<ExpressionBlock> {
    let error_pair = pair.clone();

    if pair.as_rule() == Rule::unsafe_block {
        let block_pair = pair
            .into_inner()
            .find(|inner| inner.as_rule() == Rule::block)
            .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::UnsafeBlockBody))?;

        return parse_expression_block(block_pair);
    }

    if pair.as_rule() != Rule::block {
        return Err(LoweringError::invalid(&pair, ParserIssue::ExpectedBlock));
    }

    let mut statements = Vec::new();
    let mut result = None;
    let mut inner = pair.into_inner().peekable();

    while let Some(inner_pair) = inner.next() {
        let is_last = inner.peek().is_none();

        match inner_pair.as_rule() {
            Rule::statement => {
                let is_tail_expression = is_last && !inner_pair.as_str().trim_end().ends_with(';');

                match parse_statement(inner_pair)? {
                    Statement::Expression { expression } if is_tail_expression => {
                        result = Some(Box::new(expression))
                    }
                    statement => statements.push(statement),
                }
            }
            Rule::expression
            | Rule::identifier
            | Rule::primary
            | Rule::function_call
            | Rule::string
            | Rule::number => {
                let expression = parse_expression(inner_pair)?;

                if is_last {
                    result = Some(Box::new(expression));
                } else {
                    statements.push(Statement::Expression { expression });
                }
            }
            _ => {}
        }
    }

    Ok((statements, result))
}
