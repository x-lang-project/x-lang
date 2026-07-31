use std::error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use x_ast::LocatedError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_diagnostics::{Diagnostic, Span};

#[derive(Debug)]
pub enum MirLowerError {
    Located(LocatedError<MirLowerError>),
    UnallocatedVariable(String),
    InvalidMemoryPlace(String),
    TypePromotionMismatch { from: Type, to: Type },
    MissingActiveBlockContext,
    InvalidStaticInitializer(backend_messages::StaticInitializerError),
    UnsupportedExpression(String),
    InvalidContract(String),
}

impl MirLowerError {
    pub fn into_diagnostic(self, span: Span) -> Diagnostic {
        match self {
            Self::Located(LocatedError {
                span: origin,
                error,
            }) => error.into_diagnostic(Span {
                source: span.source,
                start: origin.start,
                end: origin.end,
            }),
            MirLowerError::UnallocatedVariable(name) => {
                backend_messages::mir_unallocated_variable(name, span)
            }
            MirLowerError::InvalidMemoryPlace(place) => {
                backend_messages::mir_invalid_memory_place(place, span)
            }
            MirLowerError::TypePromotionMismatch { from, to } => {
                backend_messages::mir_type_promotion_mismatch(from, to, span)
            }
            MirLowerError::MissingActiveBlockContext => {
                backend_messages::mir_missing_active_block_context(span)
            }
            MirLowerError::InvalidStaticInitializer(detail) => {
                backend_messages::mir_invalid_static_initializer(detail.to_string(), span)
            }
            MirLowerError::UnsupportedExpression(detail) => {
                backend_messages::mir_unsupported_expression(detail, span)
            }
            MirLowerError::InvalidContract(detail) => {
                backend_messages::mir_invalid_contract(detail, span)
            }
        }
    }
}
