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

use std::error;
use std::io;
use std::{fmt as std_fmt, sync::OnceLock};

use tracing_subscriber::{EnvFilter, fmt};

use x_diagnostics::ToolingError;
use x_diagnostics::{Diagnostic, Severity, Suggestion};

static INIT: OnceLock<Result<(), InitialisationError>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialisationError(String);

impl std_fmt::Display for InitialisationError {
    fn fmt(&self, formatter: &mut std_fmt::Formatter<'_>) -> std_fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl error::Error for InitialisationError {}

pub use tracing::{Level, debug, error, info, span, trace, warn};

// Disabled trace events must not format suggestions or allocate.
fn suggestion_summaries(diag: &Diagnostic) -> Vec<String> {
    diag.suggestions
        .iter()
        .map(|suggestion: &Suggestion| {
            format!("{} edits={}", suggestion.message, suggestion.edits.len())
        })
        .collect()
}
