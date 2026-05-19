use std::error;
use std::fmt;
use std::fmt::Debug;

use crate::{
    Diagnostic, DiagnosticFact, DiagnosticFix, Error, FactKind, JsonDiagnosticPayload, Label, Span,
};

#[derive(Clone)]
pub struct TypeName {
    display: String,
    debug: String,
}
impl TypeName {
    pub fn new(display: String, debug: String) -> Self {
        Self { display, debug }
    }
}

impl fmt::Display for TypeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.display)
    }
}

impl Debug for TypeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.debug)
    }
}

#[derive(Debug, Clone)]
pub enum TypeDiagnostic {
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
        category: String,
    },
    ArityMismatch {
        expected: usize,
        got: usize,
        symbol_name: String,
    },
    UnresolvedProperty {
        field: String,
        value_type: TypeName,
    },
    InferenceConflict(String),
    InvariantFieldBorrow {
        owner: String,
    },
    UnknownIdentifier(String),
    LookupIssue(Box<super::FrontendMessage>),
    ImmutableParameterAssignment {
        name: String,
    },
    ResourceLimit {
        resource: &'static str,
        maximum: usize,
    },
    InvalidAst(String),
    UnsupportedExecutableType {
        value_type: TypeName,
        reason: String,
    },
    ConstantIndexOutOfBounds {
        index: i64,
        length: usize,
    },
}

impl TypeDiagnostic {
    pub fn display_diagnostic(&self) -> crate::Diagnostic {
        match self {
            Self::TypeMismatch { expected, got, .. } => {
                Error::new(format!("Type mismatch: expected {expected}, found {got}"))
            }
            Self::PurityViolation {
                caller,
                callee,
                missing,
                ..
            } => Error::new(format!(
                "Purity constraint violation: '{caller}' calls '{callee}' with observable operations: {missing}"
            )),
            Self::ImpureOperation {
                function,
                operation,
                category,
                ..
            } => Error::new(format!(
                "Impure operation in '{function}': {operation} is observable as '{category}'"
            )),
            Self::ArityMismatch {
                symbol_name,
                expected,
                got,
                ..
            } => Error::new(format!(
                "Arity mismatch: `{symbol_name}` expects {expected} arguments, received {got}"
            )),
            Self::UnresolvedProperty {
                field, value_type, ..
            } => Error::new(format!(
                "Unknown field or method `{field}` on `{value_type}`"
            )),
            Self::InferenceConflict(value0) => {
                Error::new(format!("Type checking failed: {value0}"))
            }
            Self::InvariantFieldBorrow { owner, .. } => Error::new(format!(
                "Cannot mutably borrow a field beneath invariant-bearing record `{owner}`: field references cannot retain the owning invariant; borrow the whole record instead"
            )),
            Self::UnknownIdentifier(value0) => Error::new(format!("Unknown identifier: {value0}")),
            Self::LookupIssue(issue) => Error::new(format!("Unknown identifier: {issue}")),
            Self::ImmutableParameterAssignment { name, .. } => Error::new(format!(
                "Cannot assign to immutable function parameter '{name}'"
            )),
            Self::ResourceLimit {
                resource, maximum, ..
            } => Error::new(format!(
                "Resource limit exceeded: {resource} exceeds the maximum of {maximum}"
            )),
            Self::InvalidAst(value0) => Error::new(format!("Invalid AST invariant: {value0}")),
            Self::UnsupportedExecutableType {
                value_type, reason, ..
            } => Error::new(format!(
                "Unsupported executable type '{value_type}': {reason}"
            )),
            Self::ConstantIndexOutOfBounds { index, length, .. } => Error::new(format!(
                "Constant array index {index} is outside the fixed length {length}"
            )),
        }
    }
}

impl fmt::Display for TypeDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.display_diagnostic().render_detail())
    }
}

impl error::Error for TypeDiagnostic {}

impl TypeDiagnostic {
    pub fn into_diagnostic(self) -> Diagnostic {
        match self {
            Self::TypeMismatch {
                got_span,
                expected_span,
                expected,
                got,
            } => Self::type_mismatch_diagnostic(got_span, expected_span, expected, got),
            Self::PurityViolation { caller, callee, missing } => Self::purity_violation_diagnostic(caller, callee, missing),
            Self::ImpureOperation {
                function,
                operation,
                category,
            } => Self::impure_operation_diagnostic(function, operation, category),
            Self::ArityMismatch {
                expected,
                got,
                symbol_name,
            } => Self::arity_mismatch_diagnostic(expected, got, symbol_name),
            Self::UnresolvedProperty { field, value_type } => Self::unresolved_property_diagnostic(field, value_type),
            Self::InferenceConflict(message) => Error::new("type checking failed")
                .with_code("E0005")
                .with_note(message),
            Self::InvariantFieldBorrow { owner } =>
                Error::new("mutable field borrow loses its owning invariant")
                    .with_code("E0005")
                    .with_note(format!(
                        "a field reference beneath `{owner}` cannot preserve the owning record invariant on later writes"
                    ))
                    .with_help("borrow the whole record instead, or use an immutable field reference"),
            Self::UnknownIdentifier(name) => Self::unknown_identifier_diagnostic(name),
            Self::LookupIssue(issue) => Self::lookup_issue_diagnostic(*issue),
            Self::ImmutableParameterAssignment { name } => Self::immutable_parameter_assignment_diagnostic(name),
            Self::ResourceLimit { resource, maximum } => Self::resource_limit_diagnostic(resource, maximum),
            Self::InvalidAst(message) => Error::new("invalid AST boundary")
                .with_code("E0012")
                .with_note(message)
                .with_help(
                    "reduce the source and report this diagnostic if it came from the X parser",
                ),
            Self::UnsupportedExecutableType { value_type, reason } =>
                Error::new("unsupported executable type")
                    .with_code("E0013")
                    .with_note(format!("`{value_type}` has no executable X 1.0 representation: {reason}"))
                    .with_help("use a fixed-size representation with a documented executable ABI"),
            Self::ConstantIndexOutOfBounds { index, length } => Self::constant_index_output_of_bounds_diagnostic(index, length),
}
    }
}

mod render;
