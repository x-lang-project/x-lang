use std::error;
use std::fmt::{self, Display, Formatter};

use x_ast::Type;
use x_diagnostics::frontend_messages;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirValidationError {
    UnresolvedType { path: String, value_type: Type },
    NonFiniteFloat { path: String },
    EmptySymbol { path: String },
}

impl Display for HirValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let issue = match self {
            Self::UnresolvedType { path, value_type } => {
                frontend_messages::HirValidationDiagnostic::UnresolvedType {
                    path: path.clone(),
                    value_type: format!("{value_type:?}"),
                }
            }
            Self::NonFiniteFloat { path } => {
                frontend_messages::HirValidationDiagnostic::NonFiniteFloat { path: path.clone() }
            }
            Self::EmptySymbol { path } => {
                frontend_messages::HirValidationDiagnostic::EmptySymbol { path: path.clone() }
            }
        };

        Display::fmt(&issue, formatter)
    }
}

impl error::Error for HirValidationError {}
