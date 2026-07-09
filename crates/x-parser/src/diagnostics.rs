use pest::error::{Error as PestError, InputLocation, LineColLocation};

use x_diagnostics::{CompilationFailure, parser_messages};

pub(super) fn parser_ice_diagnostic(detail: &str) -> CompilationFailure {
    let identifier = stable_parser_ice_identifier(detail);

    parser_messages::parser_ice(detail, &identifier)
}

pub(super) fn stable_parser_ice_identifier(detail: &str) -> String {
    let hash = x_utils::fnv1a_64(detail.as_bytes());

    parser_messages::parser_ice_identifier(hash)
}
