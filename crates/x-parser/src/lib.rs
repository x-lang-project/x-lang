#![cfg_attr(
    not(test),
    deny(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unreachable,
        clippy::unwrap_used
    )
)]

use pest::iterators::Pair;
use pest_derive::Parser;

use pipeline::EntryPoint;
use x_ast::{Expression, Program, SourceSpan};
use x_diagnostics::CompilationFailure;
use x_diagnostics::parser_messages;

mod array;
mod attributes;
mod baseline;
mod block;
mod boolean;
mod call_targets;
mod r#const;
mod cst_validation;
mod derives;
mod diagnostics;
mod expression;
mod external_function;
mod r#for;
mod function;
mod identifier;
mod r#if;
mod import;
mod integer_suffix;
mod lowering_error;
mod number;
mod pipeline;
mod postfix_operator;
mod program;
mod reference_syntax;
mod scalar;
mod script_entry;
mod source_limits;
mod worker;
pub use source_limits::MAX_SOURCE_BYTES;
mod statement;
mod string;
mod r#struct;
mod term;
mod types;
mod r#while;

#[derive(Parser)]
#[grammar = "x.pest"]
pub struct XParser;

const PARSER_STACK_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, Default)]
pub struct ParseOptions {
    pub disable_semantic_keywords: bool,
}

pub fn parse_with_options(
    input: &str,
    options: ParseOptions,
) -> Result<Program, CompilationFailure> {
    program_without_file_modules(parse_checked(input, options, EntryPoint::Script)?)
}

fn program_without_file_modules(parsed: ParsedModule) -> Result<Program, CompilationFailure> {
    if !parsed.submodules.is_empty() {
        return Err(parser_messages::file_module_project_required());
    }

    Ok(parsed.program)
}

pub fn parse_module_with_options(
    input: &str,
    options: ParseOptions,
) -> Result<Program, CompilationFailure> {
    program_without_file_modules(parse_checked(input, options, EntryPoint::Module)?)
}

pub struct ParsedModule {
    pub program: Program,
    pub item_spans: Vec<Option<SourceSpan>>,
    pub submodules: Vec<String>,
}

pub fn parse_module_with_spans(
    input: &str,
    options: ParseOptions,
) -> Result<ParsedModule, CompilationFailure> {
    parse_checked(input, options, EntryPoint::Module)
}

fn parse_checked(
    input: &str,
    options: ParseOptions,
    entrypoint: EntryPoint,
) -> Result<ParsedModule, CompilationFailure> {
    // Use a sized worker stack after bounding source nesting.
    source_limits::validate_source(input)
        .map_err(|error| error.diagnostic().into_failure(&error))?;

    worker::parse(input, options, entrypoint)
}

pub fn parse(input: &str) -> Result<Program, CompilationFailure> {
    parse_with_options(input, ParseOptions::default())
}

pub(crate) fn source_span_from_pair(pair: &Pair<Rule>) -> SourceSpan {
    let span = pair.as_span();

    SourceSpan::new(span.start(), span.end())
}

pub(crate) fn with_pair_span(mut expression: Expression, pair: &Pair<Rule>) -> Expression {
    if expression.span.is_none() {
        expression.span = Some(source_span_from_pair(pair));
    }

    expression
}
