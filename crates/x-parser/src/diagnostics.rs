use pest::error::{Error as PestError, InputLocation, LineColLocation};

use crate::Rule;
use x_diagnostics::{CompilationFailure, parser_messages};

pub(super) fn parser_ice_diagnostic(detail: &str) -> CompilationFailure {
    let identifier = stable_parser_ice_identifier(detail);

    parser_messages::parser_ice(detail, &identifier)
}

pub(super) fn stable_parser_ice_identifier(detail: &str) -> String {
    let hash = x_utils::fnv1a_64(detail.as_bytes());

    parser_messages::parser_ice_identifier(hash)
}

pub(super) fn format_syntax_error(error: PestError<Rule>) -> CompilationFailure {
    let (line, column) = match error.line_col {
        LineColLocation::Pos(position) | LineColLocation::Span(position, _) => position,
    };
    let byte = match error.location {
        InputLocation::Pos(position) | InputLocation::Span((position, _)) => position,
    };

    parser_messages::syntax_error(
        parser_messages::SyntaxPosition { line, column, byte },
        error,
    )
}
