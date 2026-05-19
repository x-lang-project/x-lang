use std::error;
use std::fmt;

use crate::{Diagnostic, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstValidationError {
    ExpressionDepth { path: String, maximum: usize },
    StatementDepth { path: String, maximum: usize },
    EmptySymbol { path: String },
    NonFiniteFloat { path: String },
    NodeCount { path: String, maximum: usize },
    ReceiverPosition { path: String },
}

impl AstValidationError {
    pub fn into_diagnostic(self) -> Diagnostic {
        match self {
            Self::ExpressionDepth { path, maximum } => Error::new(format!("expression depth at {path} exceeds the maximum of {maximum}; split it into named intermediate values")).with_code("A0101"),
            Self::StatementDepth { path, maximum } => Error::new(format!("statement depth at {path} exceeds the maximum of {maximum}; extract a helper function")).with_code("A0102"),
            Self::EmptySymbol { path } => Error::new(format!("empty symbol at {path}; reduce the input and report this diagnostic")).with_code("A0103"),
            Self::NonFiniteFloat { path } => Error::new(format!("non-finite floating-point literal at {path}")).with_code("A0104"),
            Self::ReceiverPosition { path } => Error::new(format!("receiver parameter at {path} must be the first function parameter")).with_code("A0106"),
            Self::NodeCount { maximum, path } => Error::new(format!("AST node count exceeds the maximum of {maximum} at {path}")).with_code("A0105"),
        }
    }
}

impl fmt::Display for AstValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.clone().into_diagnostic().into_failure(), formatter)
    }
}

impl error::Error for AstValidationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirValidationDiagnostic {
    UnresolvedType { path: String, value_type: String },
    NonFiniteFloat { path: String },
    EmptySymbol { path: String },
}

impl HirValidationDiagnostic {
    pub fn display_diagnostic(&self) -> crate::Diagnostic {
        match self {
            Self::UnresolvedType {
                value_type, path, ..
            } => Error::new(format!("unresolved type `{value_type}` at {path}")).with_code("H1001"),
            Self::NonFiniteFloat { path, .. } => {
                Error::new(format!("non-finite floating-point literal at {path}"))
                    .with_code("H1002")
            }
            Self::EmptySymbol { path, .. } => {
                Error::new(format!("empty symbol name at {path}")).with_code("H1003")
            }
        }
    }
}

impl fmt::Display for HirValidationDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.display_diagnostic().render_detail())
    }
}

impl error::Error for HirValidationDiagnostic {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirLowerDiagnostic {
    NonFunctionImplItem(String),
    UnsupportedStatement(String),
}

impl HirLowerDiagnostic {
    pub fn display_diagnostic(&self) -> crate::Diagnostic {
        match self {
            Self::NonFunctionImplItem(value0) => Error::new(format!(
                "expected function in impl method list, found {value0}"
            )),
            Self::UnsupportedStatement(value0) => Error::new(format!(
                "unsupported statement reached HIR lowering: {value0}"
            )),
        }
    }
}

impl fmt::Display for HirLowerDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.display_diagnostic().render_detail())
    }
}

impl error::Error for HirLowerDiagnostic {}
