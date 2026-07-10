use std::error;
use std::fmt;

use pest::iterators::Pair;

use crate::Rule;
use x_diagnostics::parser_messages::{self, ExpectedSyntax, ParserIssue};

pub(crate) type LoweringResult<T> = Result<T, LoweringError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoweringError {
    offset: usize,
    rule: Rule,
    detail: ParserIssue,
}

impl LoweringError {
    pub(crate) fn missing(pair: &Pair<Rule>, expected: ExpectedSyntax) -> Self {
        Self::invalid(pair, ParserIssue::Missing(expected))
    }

    pub(crate) fn invalid(pair: &Pair<Rule>, detail: ParserIssue) -> Self {
        Self {
            offset: pair.as_span().start(),
            rule: pair.as_rule(),
            detail,
        }
    }
}

impl fmt::Display for LoweringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&parser_messages::lowering_message(
            self.offset,
            self.rule,
            &self.detail,
        ))
    }
}

impl error::Error for LoweringError {}

pub(crate) fn required_child(
    pair: Pair<'_, Rule>,
    expected: ExpectedSyntax,
) -> LoweringResult<Pair<'_, Rule>> {
    let error_pair = pair.clone();

    pair.into_inner()
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, expected))
}
