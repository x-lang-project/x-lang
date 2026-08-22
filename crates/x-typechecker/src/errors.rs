mod rendering;
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

#[derive(Debug, Clone)]
pub enum TypeError {
    AstBoundary(frontend_messages::AstValidationError),
    Invariant(FrontendIssue),
    GenericIssue(FrontendIssue),
    InferenceIssue(Box<FrontendMessage>),
    InvalidIssue(Box<FrontendMessage>),
    LookupIssue(Box<FrontendMessage>),
    ExecutableType {
        value_type: Type,
        reason: ExecutableTypeReason,
    },
    Frontend(CompilationFailure),
    Located(LocatedError<TypeError>),

    TypeMismatch {
        got_span: Span,
        expected_span: Span,
        expected: String,
        got: String,
    },
    PurityViolation {
        caller: String,
        callee: String,
        missing: String,
    },
    ImpureOperation {
        function: String,
        operation: String,
        category: x_ast::intrinsics::EffectCategory,
    },
    ArityMismatch {
        expected: usize,
        got: usize,
        symbol_name: String,
    },
    UnresolvedProperty {
        field: String,
        value_type: Type,
    },
    InferenceConflict(String),
    InvariantFieldBorrow {
        owner: String,
    },
    UnknownIdentifier(String),
    ImmutableParameterAssignment {
        name: String,
    },
    ResourceLimit {
        resource: TypeResource,
        maximum: usize,
    },
    InvalidAst(String),
    ConstantIndexOutOfBounds {
        index: i64,
        length: usize,
    },
}

impl TypeError {
    pub fn inference(issue: FrontendMessage) -> Self {
        Self::InferenceIssue(Box::new(issue))
    }

    pub fn invalid(issue: FrontendMessage) -> Self {
        Self::InvalidIssue(Box::new(issue))
    }

    pub fn lookup(issue: FrontendMessage) -> Self {
        Self::LookupIssue(Box::new(issue))
    }
}

x_ast::impl_source_error!(TypeError);

impl Display for TypeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Located(location) => Display::fmt(&location.error, formatter),
            Self::Frontend(failure) => Display::fmt(failure, formatter),
            other => Display::fmt(&other.clone().into_catalog(), formatter),
        }
    }
}

impl error::Error for TypeError {}
