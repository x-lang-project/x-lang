mod resources;

pub use resources::TypeResource;

use std::error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::Path;

use x_ast::LocatedError;
use x_ast::Type;
use x_diagnostics::CompilationFailure;
use x_diagnostics::frontend_messages;
use x_diagnostics::frontend_messages::{
    ExecutableTypeReason, FrontendIssue, FrontendMessage, TypeDiagnostic, TypeName,
};
use x_diagnostics::{Diagnostic, SourceIdentifier, Span};
