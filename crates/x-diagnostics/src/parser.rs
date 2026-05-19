use std::fmt;

use crate::{CompilationFailure, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserDiagnostic {
    FileModuleProjectRequired,
    EmptyInput,
    AstValidation,
    Syntax,
    StatementDeclarationMissing,
    ModuleNameMissing,
    StatementLowering,
    UnexpectedProgramRule,
    SourceTooLarge,
    NestingTooDeep,
    NumericLiteralTooLarge,
    StringLiteralTooLarge,
    NonFiniteFloat,
    IntegerOutOfRange,
    TooManyTokens,
    TokenTooLarge,
    PrefixChainTooDeep,
    ScannerDidNotAdvance,
    DelimiterPrefixTooDeep,
    MissingCstChild,
    UnexpectedCstChild,
    UnsupportedCstShape,
    ImportPathEmpty,
    ImportUnexpectedRule,
}

impl ParserDiagnostic {
    pub const fn code(self) -> &'static str {
        match self {
            Self::FileModuleProjectRequired | Self::StatementLowering => "P0204",
            Self::EmptyInput | Self::ModuleNameMissing => "XPR0001",
            Self::AstValidation => "P0301",
            Self::Syntax | Self::UnexpectedProgramRule => "P0200",
            Self::StatementDeclarationMissing | Self::MissingCstChild => "P0201",
            Self::SourceTooLarge => "P0100",
            Self::NestingTooDeep => "P0101",
            Self::NumericLiteralTooLarge => "P0102",
            Self::StringLiteralTooLarge => "P0103",
            Self::NonFiniteFloat => "P0104",
            Self::IntegerOutOfRange => "P0105",
            Self::TooManyTokens => "P0106",
            Self::TokenTooLarge => "P0107",
            Self::PrefixChainTooDeep => "P0108",
            Self::ScannerDidNotAdvance => "P0109",
            Self::DelimiterPrefixTooDeep => "P0110",
            Self::UnexpectedCstChild => "P0202",
            Self::UnsupportedCstShape => "P0203",
            Self::ImportPathEmpty => "P0212",
            Self::ImportUnexpectedRule => "P0213",
        }
    }

    pub fn into_failure(self, detail: impl fmt::Display) -> CompilationFailure {
        Error::new(detail.to_string())
            .with_code(self.code())
            .into_failure()
    }
}
