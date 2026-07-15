use std::error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use x_ast::ExpressionKind;
use x_ast::LocatedError;
use x_diagnostics::borrow_messages;
use x_diagnostics::borrow_messages::{BorrowDiagnostic, DebugValue};
use x_diagnostics::{Diagnostic, Span};

#[derive(Debug, Clone)]
pub enum BorrowCheckError {
    Invariant(borrow_messages::BorrowInvariant),
    Located(LocatedError<BorrowCheckError>),
    UndeclaredVariable(String),
    UseAfterMove(String),
    UseBeforeInitialisation(String),
    MoveWhileBorrowed(String),
    MutableBorrowViolation(String),
    ImmutableBorrowViolation(String),
    LifetimeTooShort { target: String },
    DanglingBranchReference,
    InvalidReferenceReturn,
    UnknownReferenceOrigin { target: String },
    InvalidAssignmentLValue,
    NonLValueReference(ExpressionKind),
    InternalStateCorruption(String),
}

impl BorrowCheckError {
    fn into_catalog(self) -> BorrowDiagnostic {
        match self {
            Self::Invariant(issue) => BorrowDiagnostic::InternalStateCorruption(issue.to_string()),
            Self::Located(location) => location.error.into_catalog(),
            Self::UndeclaredVariable(value) => BorrowDiagnostic::UndeclaredVariable(value),
            Self::UseAfterMove(value) => BorrowDiagnostic::UseAfterMove(value),
            Self::UseBeforeInitialisation(value) => {
                BorrowDiagnostic::UseBeforeInitialisation(value)
            }
            Self::MoveWhileBorrowed(value) => BorrowDiagnostic::MoveWhileBorrowed(value),
            Self::MutableBorrowViolation(value) => BorrowDiagnostic::MutableBorrowViolation(value),
            Self::ImmutableBorrowViolation(value) => {
                BorrowDiagnostic::ImmutableBorrowViolation(value)
            }
            Self::LifetimeTooShort { target } => BorrowDiagnostic::LifetimeTooShort { target },
            Self::DanglingBranchReference => BorrowDiagnostic::DanglingBranchReference,
            Self::InvalidReferenceReturn => BorrowDiagnostic::InvalidReferenceReturn,
            Self::UnknownReferenceOrigin { target } => {
                BorrowDiagnostic::UnknownReferenceOrigin { target }
            }
            Self::InvalidAssignmentLValue => BorrowDiagnostic::InvalidAssignmentLValue,
            Self::NonLValueReference(value) => {
                BorrowDiagnostic::NonLValueReference(DebugValue(format!("{value:?}")))
            }
            Self::InternalStateCorruption(value) => {
                BorrowDiagnostic::InternalStateCorruption(value)
            }
        }
    }
}

impl Display for BorrowCheckError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Located(location) => Display::fmt(&location.error, formatter),
            other => Display::fmt(&other.clone().into_catalog(), formatter),
        }
    }
}

impl error::Error for BorrowCheckError {}
