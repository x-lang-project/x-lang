mod bindings;
use pest::iterators::Pair;

use crate::block;
use crate::external_function;
use crate::import;
use crate::{
    Rule,
    r#const::parse_const_declaration,
    expression::parse_expression,
    external_function::parse_external_function_declaration,
    function::{parse_function_call, parse_function_definition},
    lowering_error::{LoweringError, LoweringResult, required_child},
    term::parse_term,
    types::parse_type,
};
use bindings::*;
use declarations::parse_type_alias_declaration;
use x_ast::{Expression, ExpressionKind, Pattern, Statement, Type};
use x_diagnostics::parser_messages::ExpectedSyntax;
use x_diagnostics::parser_messages::ParserIssue;

mod declarations;
pub(crate) mod generics;

pub use declarations::{parse_impl_definition, parse_trait_declaration};

pub fn parse_statement(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let statement = required_child(pair, ExpectedSyntax::Statement)?;

    let rule = statement.as_rule();
    let statement = unwrap_statement(statement)?;

    match rule {
        Rule::if_statement => crate::r#if::parse_if_statement(statement),
        Rule::variable_declaration_statement => parse_variable_declaration(statement),
        Rule::const_declaration_statement => parse_const_declaration(statement),
        Rule::type_alias_declaration => parse_type_alias_declaration(statement),
        Rule::import_statement => parse_import_statement(statement, &error_pair),
        Rule::mod_declaration => parse_module_statement(statement, &error_pair),
        Rule::return_statement => parse_return_statement(statement),
        Rule::throw_statement => parse_throw_statement(statement),
        Rule::panic_statement => parse_panic_statement(statement),
        Rule::become_statement => parse_become_statement(statement),
        Rule::break_statement => Ok(Statement::Break),
        Rule::continue_statement => Ok(Statement::Continue),
        Rule::function_definition => parse_function_definition(statement),
        Rule::struct_declaration => crate::r#struct::parse_struct_declaration(statement),
        Rule::for_each_loop => crate::r#for::parse_for_each_loop(statement),
        Rule::for_range_loop => crate::r#for::parse_for_range_loop(statement),
        Rule::while_loop => crate::r#while::parse_while_loop(statement),
        Rule::loop_statement => crate::r#while::parse_infinite_loop(statement),
        Rule::invariant_statement => parse_invariant_statement(statement),
        Rule::unsafe_statement | Rule::block => block::parse_block(statement),
        Rule::assignment_statement | Rule::assignment => parse_assignment_statement(statement),
        Rule::trait_declaration => parse_trait_declaration(statement),
        Rule::impl_definition => parse_impl_definition(statement),
        Rule::expression_statement => parse_expression_statement(statement),
        Rule::match_expression => expression_statement(statement),
        Rule::external_function_declaration => parse_external_function_declaration(statement),
        Rule::external_block => external_function::parse_external_block(statement),
        Rule::static_declaration => crate::r#const::parse_static_declaration(statement),
        Rule::do_statement => parse_call_statement(statement),
        _ => Err(LoweringError::invalid(
            &statement,
            ParserIssue::UnsupportedStatementRule,
        )),
    }
}

fn unwrap_statement(pair: Pair<Rule>) -> LoweringResult<Pair<Rule>> {
    let expected = match pair.as_rule() {
        Rule::variable_declaration_statement => ExpectedSyntax::VariableDeclaration,
        Rule::const_declaration_statement => ExpectedSyntax::ConstantDeclaration,
        Rule::unsafe_statement => ExpectedSyntax::UnsafeBlock,
        Rule::assignment_statement | Rule::assignment => ExpectedSyntax::Assignment,
        _ => return Ok(pair),
    };

    required_child(pair, expected)
}

fn expression_statement(pair: Pair<Rule>) -> LoweringResult<Statement> {
    Ok(Statement::Expression {
        expression: parse_expression(pair)?,
    })
}

fn parse_import_statement(pair: Pair<Rule>, error_pair: &Pair<Rule>) -> LoweringResult<Statement> {
    pair.into_inner()
        .next()
        .ok_or(import::ImportLoweringError::EmptyPath)
        .and_then(import::parse_import)
        .map_err(|error| {
            LoweringError::invalid(error_pair, ParserIssue::Upstream(error.to_string()))
        })
}

fn parse_module_statement(pair: Pair<Rule>, error_pair: &Pair<Rule>) -> LoweringResult<Statement> {
    let name = pair
        .into_inner()
        .next()
        .ok_or_else(|| LoweringError::missing(error_pair, ExpectedSyntax::ModuleName))?;

    Ok(Statement::Comment(format!("mod {}", name.as_str())))
}

fn parse_call_statement(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let call = required_child(pair, ExpectedSyntax::FunctionCall)?;

    Ok(Statement::Expression {
        expression: parse_function_call(call)?,
    })
}

fn parse_invariant_statement(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let invariant = required_child(pair, ExpectedSyntax::Invariant)?;
    let body = invariant
        .into_inner()
        .find(|child| child.as_rule() == Rule::block)
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::InvariantBody))?;
    let statements = block::parse_block_statements(body)?;

    // Block lowering represents tail predicates as returns.
    let condition = match statements.as_slice() {
        [Statement::Expression { expression }]
        | [
            Statement::Return {
                value: Some(expression),
            },
        ] => expression.clone(),
        _ => Expression {
            span: None,
            value_type: Type::Boolean,
            kind: ExpressionKind::Boolean(false),
        },
    };

    Ok(Statement::Assert {
        condition,
        message: None,
    })
}

fn parse_expression_statement(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let expression = pair
        .into_inner()
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::Expression))?;

    expression_statement(expression)
}

fn parse_become_statement(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let call_pair = pair
        .into_inner()
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::BecomeCall))?;

    Ok(Statement::Become {
        call: parse_function_call(call_pair)?,
    })
}

fn parse_return_statement(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let value = pair.into_inner().next().map(parse_expression).transpose()?;

    Ok(Statement::Return { value })
}

fn parse_panic_statement(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let message = pair
        .into_inner()
        .find(|part| part.as_rule() == Rule::expression)
        .map(parse_expression)
        .transpose()?;

    Ok(Statement::Panic { message })
}

fn parse_throw_statement(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let message = pair
        .into_inner()
        .find(|part| part.as_rule() == Rule::expression)
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::ThrownExpression))?;

    Ok(Statement::Throw {
        message: parse_expression(message)?,
    })
}
