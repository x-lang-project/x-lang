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

mod pipeline;

mod worker;

#[derive(Debug, Clone, Copy, Default)]
pub struct ParseOptions {
    pub disable_semantic_keywords: bool,
}
