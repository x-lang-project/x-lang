use std::error;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::{Diagnostic, DiagnosticFact, DiagnosticFix, Error, FactKind, Label, Span};

#[derive(Clone)]
pub struct DebugValue(pub String);
impl Debug for DebugValue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
pub enum BorrowDiagnostic {
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
    NonLValueReference(DebugValue),
    InternalStateCorruption(String),
}

impl BorrowDiagnostic {
    pub fn display_diagnostic(&self) -> crate::Diagnostic {
        match self {
            Self::UndeclaredVariable(value0) => {
                Error::new(format!("undeclared identifier '{value0}'"))
            }
            Self::UseAfterMove(value0) => Error::new(format!(
                "Variable '{value0}' was moved out and cannot be used here"
            )),
            Self::UseBeforeInitialisation(value0) => Error::new(format!(
                "Variable '{value0}' may be uninitialised when used here"
            )),
            Self::MoveWhileBorrowed(value0) => Error::new(format!(
                "Cannot move out of '{value0}' because it is borrowed"
            )),
            Self::MutableBorrowViolation(value0) => Error::new(format!(
                "Cannot borrow '{value0}' as mutable because it is already borrowed"
            )),
            Self::ImmutableBorrowViolation(value0) => Error::new(format!(
                "Cannot borrow '{value0}' as immutable because it is already mutably borrowed"
            )),
            Self::LifetimeTooShort { target, .. } => {
                Error::new(format!("reference to '{target}' outlives its scope"))
            }
            Self::DanglingBranchReference => {
                Error::new("if branch returns a reference that does not outlive the branch")
            }
            Self::InvalidReferenceReturn => {
                Error::new("cannot return a reference from this function")
            }
            Self::UnknownReferenceOrigin { target, .. } => Error::new(format!(
                "Cannot establish reference origins for '{target}' before writing through a borrow"
            )),
            Self::InvalidAssignmentLValue => {
                Error::new("field assignment requires a resolvable owner")
            }
            Self::NonLValueReference(value0) => Error::new(format!(
                "Cannot perform operation on non-lvalue expression: {value0:?}"
            )),
            Self::InternalStateCorruption(value0) => {
                Error::new(format!("internal borrow checker error: {value0}"))
            }
        }
    }
}

impl fmt::Display for BorrowDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.display_diagnostic().render_detail())
    }
}

impl error::Error for BorrowDiagnostic {}

impl BorrowDiagnostic {
    pub fn into_diagnostic(self, span: Span) -> Diagnostic {
        match self {
            Self::UndeclaredVariable(name) => Self::undeclared_variable_diagnostic(name, span),
            Self::UseAfterMove(name) => Self::use_after_move_diagnostic(name, span),
            Self::UseBeforeInitialisation(name) => Self::use_before_initialise_diagnostic(name, span),
            Self::MoveWhileBorrowed(name) => Self::move_while_borrowed_diagnostic(name, span),
            Self::MutableBorrowViolation(name) => Self::mutable_borrow_violation_diagnostic(name, span),
            Self::ImmutableBorrowViolation(name) => Self::immutable_borrow_violation_diagnostic(name, span),
            Self::LifetimeTooShort { target } => Self::lifetime_too_short_diagnostic(target, span),
            Self::DanglingBranchReference => {
                Error::new("branch returns a dangling reference")
                    .with_code("B0007")
                    .with_label(Label::primary(
                        span,
                        "this branch can produce a reference that expires too soon",
                    ))
                    .with_help("return an owned value from the branch")
            }
            Self::UnknownReferenceOrigin { target } => {
                Error::new("unknown reference origin")
                    .with_code("B0012")
                    .with_label(Label::primary(span, format!("cannot establish reference origins for `{target}` before this borrowed store")))
            }
            Self::InvalidReferenceReturn => {
                Error::new("cannot return reference from this function")
                    .with_code("B0008")
                    .with_label(Label::primary(
                        span,
                        "reference return would escape the current function",
                    ))
                    .with_help("return an owned value or specify the reference lifetime")
            }
            Self::InvalidAssignmentLValue => {
                Error::new("invalid assignment target")
                    .with_code("B0009")
                    .with_label(Label::primary(span, "this expression is not assignable"))
                    .with_help("assign to a local, field, array slot, or dereferenceable pointer")
            }
            Self::NonLValueReference(kind) => Self::non_l_value_reference_diagnostic(kind, span),
            Self::InternalStateCorruption(detail) => {
                Error::new("internal borrow checker error")
                    .with_code("B9999")
                    .with_label(Label::primary(span, detail))
                    .with_help("this is a compiler bug; reduce the input and report it")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorrowInvariant {
    ScopeStackExecutionUnderflow,
    AttemptedToPopBaseCompilationFrame,
    MissingVariableAllocationScopeContext,
    ParameterAggregateLayoutDepthExceeded { maximum: usize },
    StoredAggregateLayoutDepthExceeded { maximum: usize },
    UnallocatedScopeContext,
}
impl Display for BorrowInvariant {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScopeStackExecutionUnderflow => {
                formatter.write_str(&Error::new("scope stack underflow").render_detail())
            }
            Self::AttemptedToPopBaseCompilationFrame => {
                formatter.write_str(&Error::new("cannot pop the base scope").render_detail())
            }
            Self::MissingVariableAllocationScopeContext => {
                formatter.write_str(&Error::new("missing variable scope").render_detail())
            }
            Self::ParameterAggregateLayoutDepthExceeded { maximum } => formatter.write_str(
                &Error::new(format!(
                    "parameter aggregate layout exceeds {maximum} levels"
                ))
                .render_detail(),
            ),
            Self::StoredAggregateLayoutDepthExceeded { maximum } => formatter.write_str(
                &Error::new(format!("stored aggregate layout exceeds {maximum} levels"))
                    .render_detail(),
            ),
            Self::UnallocatedScopeContext => {
                formatter.write_str(&Error::new("missing scope").render_detail())
            }
        }
    }
}

mod render;
