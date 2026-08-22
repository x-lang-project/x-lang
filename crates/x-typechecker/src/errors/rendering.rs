use super::*;

impl TypeError {
    pub(super) fn into_catalog(self) -> TypeDiagnostic {
        match self {
            Self::AstBoundary(error) => TypeDiagnostic::InvalidAst(error.to_string()),
            Self::Invariant(issue) => TypeDiagnostic::InvalidAst(issue.to_string()),
            Self::GenericIssue(issue) => TypeDiagnostic::InferenceConflict(issue.to_string()),
            Self::InferenceIssue(issue) => TypeDiagnostic::InferenceConflict(issue.to_string()),
            Self::InvalidIssue(issue) => TypeDiagnostic::InvalidAst(issue.to_string()),
            Self::LookupIssue(issue) => TypeDiagnostic::LookupIssue(issue),
            Self::ExecutableType { value_type, reason } => {
                TypeDiagnostic::UnsupportedExecutableType {
                    value_type: TypeName::new(value_type.to_string(), format!("{value_type:?}")),
                    reason: reason.to_string(),
                }
            }
            Self::Located(location) => location.error.into_catalog(),
            Self::Frontend(failure) => TypeDiagnostic::InferenceConflict(failure.to_string()),
            Self::TypeMismatch {
                got_span,
                expected_span,
                expected,
                got,
            } => TypeDiagnostic::TypeMismatch {
                got_span,
                expected_span,
                expected,
                got,
            },
            Self::PurityViolation {
                caller,
                callee,
                missing,
            } => TypeDiagnostic::PurityViolation {
                caller,
                callee,
                missing,
            },

            Self::ImpureOperation {
                function,
                operation,
                category,
            } => TypeDiagnostic::ImpureOperation {
                function,
                operation,
                category: category.label().to_string(),
            },
            Self::ArityMismatch {
                expected,
                got,
                symbol_name,
            } => TypeDiagnostic::ArityMismatch {
                expected,
                got,
                symbol_name,
            },
            Self::UnresolvedProperty { field, value_type } => TypeDiagnostic::UnresolvedProperty {
                field,
                value_type: TypeName::new(value_type.to_string(), format!("{value_type:?}")),
            },
            Self::InferenceConflict(value) => TypeDiagnostic::InferenceConflict(value),
            Self::InvariantFieldBorrow { owner } => TypeDiagnostic::InvariantFieldBorrow { owner },
            Self::UnknownIdentifier(value) => TypeDiagnostic::UnknownIdentifier(value),
            Self::ImmutableParameterAssignment { name } => {
                TypeDiagnostic::ImmutableParameterAssignment { name }
            }

            Self::ResourceLimit { resource, maximum } => TypeDiagnostic::ResourceLimit {
                resource: resource.label(),
                maximum,
            },
            Self::InvalidAst(value) => TypeDiagnostic::InvalidAst(value),
            Self::ConstantIndexOutOfBounds { index, length } => {
                TypeDiagnostic::ConstantIndexOutOfBounds { index, length }
            }
        }
    }

    pub fn into_diagnostic(self, source_identifier: SourceIdentifier) -> Diagnostic {
        match self {
            Self::Frontend(failure) => failure.as_diagnostic().clone(),
            Self::Located(LocatedError { span, error }) => {
                let mut diagnostic = error.into_diagnostic(source_identifier);
                let location = Span {
                    source: source_identifier,
                    start: span.start,
                    end: span.end,
                };

                if diagnostic.labels.is_empty() {
                    diagnostic
                        .labels
                        .push(frontend_messages::source_error_label(location));
                } else {
                    for label in &mut diagnostic.labels {
                        if label.span.start == 0 && label.span.end <= 1 {
                            label.span = location;
                        }
                    }
                }

                for fact in &mut diagnostic.facts {
                    if fact
                        .span
                        .is_some_and(|span| span.start == 0 && span.end <= 1)
                    {
                        fact.span = Some(location);
                    }
                }

                diagnostic
            }
            other => other.into_catalog().into_diagnostic(),
        }
    }
}
