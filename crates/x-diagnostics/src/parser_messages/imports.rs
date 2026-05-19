use std::fmt;

use crate::{CompilationFailure, Error, ParserDiagnostic};

pub fn import_path_empty() -> CompilationFailure {
    Error::new("import path is empty")
        .with_code(ParserDiagnostic::ImportPathEmpty.code())
        .into_failure()
}

pub fn import_unexpected_rule(rule: impl fmt::Debug) -> CompilationFailure {
    Error::new(format!(
        "import lowering received unexpected `{rule:?}` syntax"
    ))
    .with_code(ParserDiagnostic::ImportUnexpectedRule.code())
    .into_failure()
}
