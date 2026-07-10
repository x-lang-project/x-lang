use std::fmt;

use pest::iterators::Pair;

use crate::Rule;
use x_ast::Statement;
use x_diagnostics::parser_messages;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportLoweringError {
    EmptyPath,
    UnexpectedRule(Rule),
}

impl fmt::Display for ImportLoweringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let failure = match self {
            Self::EmptyPath => parser_messages::import_path_empty(),
            Self::UnexpectedRule(rule) => parser_messages::import_unexpected_rule(rule),
        };

        fmt::Display::fmt(&failure, formatter)
    }
}

pub fn parse_import(pair: Pair<Rule>) -> Result<Statement, ImportLoweringError> {
    match pair.as_rule() {
        Rule::import_path => {
            let (module, item) = split_import_path(pair)?;

            Ok(Statement::Import { module, item })
        }
        other => Err(ImportLoweringError::UnexpectedRule(other)),
    }
}

fn split_import_path(path: Pair<Rule>) -> Result<(String, String), ImportLoweringError> {
    normalize_import_segments(
        path.into_inner()
            .map(|segment| segment.as_str().to_string())
            .collect(),
    )
}

pub(crate) fn normalize_import_segments(
    mut segments: Vec<String>,
) -> Result<(String, String), ImportLoweringError> {
    if segments.is_empty() {
        return Err(ImportLoweringError::EmptyPath);
    }

    if segments.first().is_some_and(|segment| segment == "crate") {
        segments.remove(0);
    }

    let item = segments.pop().ok_or(ImportLoweringError::EmptyPath)?;
    let module = segments.join("::");

    Ok((module, item))
}
