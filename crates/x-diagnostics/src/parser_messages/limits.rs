use std::fmt;

use crate::{Error, ParserDiagnostic};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceLimitError {
    SourceTooLarge { actual: usize, maximum: usize },
    NestingTooDeep { offset: usize, maximum: usize },
    NumericLiteralTooLarge { offset: usize, maximum: usize },
    StringLiteralTooLarge { offset: usize, maximum: usize },
    NonFiniteFloat { offset: usize },
    IntegerOutOfRange { offset: usize },
    TooManyTokens { offset: usize, maximum: usize },
    TokenTooLarge { offset: usize, maximum: usize },
    PrefixChainTooDeep { offset: usize, maximum: usize },
    ScannerDidNotAdvance { offset: usize },
    DelimiterPrefixTooDeep { offset: usize, maximum: usize },
}

impl fmt::Display for SourceLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceTooLarge { actual, maximum } => formatter.write_str(&Error::new(format!("source is {actual} bytes; the maximum supported size is {maximum} bytes")).render_detail()),
            Self::NestingTooDeep { offset, maximum } => formatter.write_str(&Error::new(format!("nesting at byte {offset} exceeds the maximum depth of {maximum}")).render_detail()),
            Self::NumericLiteralTooLarge { offset, maximum } => formatter.write_str(&Error::new(format!("numeric literal at byte {offset} exceeds {maximum} bytes")).render_detail()),
            Self::StringLiteralTooLarge { offset, maximum } => {
                formatter.write_str(&Error::new(format!("string literal at byte {offset} exceeds {maximum} bytes")).render_detail())
            }
            Self::NonFiniteFloat { offset } => {
                formatter.write_str(&Error::new(format!("floating-point literal at byte {offset} is not finite")).render_detail())
            }
            Self::IntegerOutOfRange { offset } => formatter.write_str(&Error::new(format!("integer literal at byte {offset} is outside the supported 64-bit range")).render_detail()),
            Self::TooManyTokens { offset, maximum } => formatter.write_str(&Error::new(format!("token count exceeds the maximum of {maximum} at byte {offset}")).render_detail()),
            Self::TokenTooLarge { offset, maximum } => {
                formatter.write_str(&Error::new(format!("token at byte {offset} exceeds {maximum} bytes")).render_detail())
            }
            Self::PrefixChainTooDeep { offset, maximum } => formatter.write_str(&Error::new(format!("prefix-operator chain at byte {offset} exceeds the maximum depth of {maximum}")).render_detail()),
            Self::ScannerDidNotAdvance { offset } => {
                formatter.write_str(&Error::new(format!("source scanner made no progress at byte {offset}")).render_detail())
            }
            Self::DelimiterPrefixTooDeep { offset, maximum } => formatter.write_str(&Error::new(format!("opening-delimiter chain at byte {offset} exceeds the maximum depth of {maximum}")).render_detail()),
        }
    }
}

impl SourceLimitError {
    pub fn diagnostic(&self) -> ParserDiagnostic {
        match self {
            Self::SourceTooLarge { .. } => ParserDiagnostic::SourceTooLarge,
            Self::NestingTooDeep { .. } => ParserDiagnostic::NestingTooDeep,
            Self::NumericLiteralTooLarge { .. } => ParserDiagnostic::NumericLiteralTooLarge,
            Self::StringLiteralTooLarge { .. } => ParserDiagnostic::StringLiteralTooLarge,
            Self::NonFiniteFloat { .. } => ParserDiagnostic::NonFiniteFloat,
            Self::IntegerOutOfRange { .. } => ParserDiagnostic::IntegerOutOfRange,
            Self::TooManyTokens { .. } => ParserDiagnostic::TooManyTokens,
            Self::TokenTooLarge { .. } => ParserDiagnostic::TokenTooLarge,
            Self::PrefixChainTooDeep { .. } => ParserDiagnostic::PrefixChainTooDeep,
            Self::ScannerDidNotAdvance { .. } => ParserDiagnostic::ScannerDidNotAdvance,
            Self::DelimiterPrefixTooDeep { .. } => ParserDiagnostic::DelimiterPrefixTooDeep,
        }
    }
}
