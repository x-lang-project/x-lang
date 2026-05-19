use std::borrow;
use std::fmt;

use super::ExpectedSyntax;
use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserIssue {
    InvalidElseBody,
    ExpectedBlock,
    ExpectedFunctionParameterList,
    ExpectedUnaryExpression,
    ExpectedLvalue,
    ExternalBlockFunctionChild,
    ExternalBlockSelfParameter,
    ExternalBlockDeclaration,
    InvalidIfLowering,
    InvalidImplDeclaration,
    IncompleteCharacterEscape,
    InvalidUnicodeScalarValue,
    UnsupportedLayout,
    DuplicateLayout,
    DuplicateStructInvariant,
    InvalidStructInvariant,
    InvalidLvalueShape,
    MalformedAttribute,
    NestedTuplePattern,
    NonFiniteNumericLiteral,
    InvalidPrimaryLvalue,
    UnsupportedTermRule,
    UnsupportedTypeRule,
    UnsupportedExpressionRule,
    InvalidSimdArguments,
    MissingStringQuotes,
    InvalidStructAttributeSyntax,
    InvalidTraitMethodLowering,
    UnsupportedUninitializedBinding,
    UnknownBinaryOperatorRule,
    UnknownPrefixUnaryOperator,
    UnsupportedDerive,
    UnsupportedFunctionAttribute,
    UnsupportedFunctionParameterShape,
    UnsupportedGenericTypeArgument,
    UnsupportedMatchArmBody,
    UnsupportedPatternRule,
    UnsupportedPostfixOperation,
    UnsupportedStatementRule,
    UnsupportedStructAttribute,
    UnsupportedUnaryExpressionChild,
    UnsupportedVariableBindingPattern,
    Missing(ExpectedSyntax),
    InvalidArrayLength(String),
    InvalidNumericLiteral(String),
    UnsupportedWideInteger,
    UnknownPrimitiveType(String),
    Upstream(String),
}
impl fmt::Display for ParserIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message: borrow::Cow<'_, str> = match self {
            Self::InvalidElseBody => "else body must be a block or nested if".into(),
            Self::ExpectedBlock => "expected a block".into(),
            Self::ExpectedFunctionParameterList => "expected a function parameter list".into(),
            Self::ExpectedUnaryExpression => "expected a unary expression".into(),
            Self::ExpectedLvalue => "expected an lvalue".into(),
            Self::ExternalBlockFunctionChild => "an external block function child".into(),
            Self::ExternalBlockSelfParameter => {
                "external block functions cannot have a `self` parameter".into()
            }
            Self::ExternalBlockDeclaration => {
                "an external block may contain only external function declarations".into()
            }
            Self::InvalidIfLowering => "if lowering returned a non-if statement".into(),
            Self::InvalidImplDeclaration => {
                "impl must contain a trait or inherent declaration".into()
            }
            Self::IncompleteCharacterEscape => "incomplete character escape".into(),
            Self::InvalidUnicodeScalarValue => "invalid Unicode scalar value".into(),
            Self::UnsupportedLayout => "layout must be AoS, SoA or Auto".into(),
            Self::DuplicateLayout => "a struct may have only one layout attribute".into(),
            Self::DuplicateStructInvariant => "a struct may have only one invariant block".into(),
            Self::InvalidStructInvariant => {
                "a struct invariant must contain exactly one predicate expression".into()
            }
            Self::InvalidLvalueShape => {
                "lvalue must contain an address, dereference, or primary lvalue".into()
            }
            Self::MalformedAttribute => "malformed attribute".into(),
            Self::NestedTuplePattern => "nested tuple patterns are not supported".into(),
            Self::NonFiniteNumericLiteral => "numeric literal is not finite".into(),
            Self::InvalidPrimaryLvalue => {
                "primary lvalue must contain a postfix expression or nested lvalue".into()
            }
            Self::UnsupportedTermRule => "rule cannot be lowered as a term".into(),
            Self::UnsupportedTypeRule => "rule cannot be lowered as a type".into(),
            Self::UnsupportedExpressionRule => "rule cannot be lowered as an expression".into(),
            Self::InvalidSimdArguments => {
                "SIMD types require exactly one element type and one integer lane count".into()
            }
            Self::MissingStringQuotes => "string literal is missing quotes".into(),
            Self::InvalidStructAttributeSyntax => {
                "struct attributes must use layout(...) or derive(...)".into()
            }
            Self::InvalidTraitMethodLowering => {
                "trait method lowering returned a non-function statement".into()
            }
            Self::UnsupportedUninitializedBinding => {
                "uninitialised bindings are unsupported by the AST".into()
            }
            Self::UnknownBinaryOperatorRule => "unknown binary operator rule".into(),
            Self::UnknownPrefixUnaryOperator => "unknown prefix unary operator".into(),
            Self::UnsupportedDerive => {
                "unsupported derive; expected Debug, Clone, Default, Eq, PartialEq or Hash".into()
            }
            Self::UnsupportedFunctionAttribute => {
                "unsupported function attribute; expected memoise or no_mangle".into()
            }
            Self::UnsupportedFunctionParameterShape => {
                "unsupported function parameter shape".into()
            }
            Self::UnsupportedGenericTypeArgument => "unsupported generic type argument".into(),
            Self::UnsupportedMatchArmBody => "unsupported match arm body".into(),
            Self::UnsupportedPatternRule => "unsupported pattern rule".into(),
            Self::UnsupportedPostfixOperation => "unsupported postfix operation".into(),
            Self::UnsupportedStatementRule => "unsupported statement rule".into(),
            Self::UnsupportedStructAttribute => {
                "unsupported struct attribute; use layout(...) or derive(...)".into()
            }
            Self::UnsupportedUnaryExpressionChild => "unsupported unary expression child".into(),
            Self::UnsupportedVariableBindingPattern => {
                "unsupported variable binding pattern".into()
            }
            Self::Missing(expected) => format!("missing {expected}").into(),
            Self::InvalidArrayLength(error) => format!("invalid array length: {error}").into(),
            Self::InvalidNumericLiteral(error) => {
                format!("invalid numeric literal: {error}").into()
            }
            Self::UnsupportedWideInteger => {
                "128-bit integer types are not executable in X 1.0".into()
            }
            Self::UnknownPrimitiveType(name) => format!("unknown primitive type `{name}`").into(),
            Self::Upstream(message) => return formatter.write_str(message),
        };

        formatter.write_str(&Error::new(message).render_detail())
    }
}

pub fn lowering_message(offset: usize, rule: impl fmt::Debug, issue: &ParserIssue) -> String {
    Error::new(format!(
        "CST lowering failed at byte {offset} for `{rule:?}`: {issue}"
    ))
    .render_detail()
}

pub fn unsupported_integer_suffix() -> ParserIssue {
    ParserIssue::InvalidNumericLiteral("unsupported integer suffix".to_owned())
}

pub fn integer_representation_overflow() -> ParserIssue {
    ParserIssue::InvalidNumericLiteral("integer exceeds the AST integer representation".to_owned())
}
