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

mod integer_suffix;

mod pipeline;

mod scalar;
mod script_entry;
mod source_limits;
mod worker;
pub use source_limits::MAX_SOURCE_BYTES;
mod statement;

#[derive(Debug, Clone, Copy, Default)]
pub struct ParseOptions {
    pub disable_semantic_keywords: bool,
}

pub struct ParsedModule {
    pub program: Program,
    pub item_spans: Vec<Option<SourceSpan>>,
    pub submodules: Vec<String>,
}
