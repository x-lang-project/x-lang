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
