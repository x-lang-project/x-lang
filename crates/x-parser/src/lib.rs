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

mod derives;
mod diagnostics;

mod integer_suffix;

mod number;
mod pipeline;

mod reference_syntax;
mod scalar;
mod script_entry;
mod source_limits;
mod worker;
pub use source_limits::MAX_SOURCE_BYTES;
mod statement;

#[derive(Parser)]
#[grammar = "x.pest"]
pub struct XParser;

const PARSER_STACK_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, Default)]
pub struct ParseOptions {
    pub disable_semantic_keywords: bool,
}

pub struct ParsedModule {
    pub program: Program,
    pub item_spans: Vec<Option<SourceSpan>>,
    pub submodules: Vec<String>,
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
