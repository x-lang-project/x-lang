use std::fmt;

use crate::{CompilationFailure, Error, Label, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontendIssue {
    MissingIndexedOverload {
        name: String,
    },
    MonomorphisationQueueChanged,
    UnknownGenericStruct {
        name: String,
    },
    MissingGenericParameters {
        name: String,
    },
    GenericStructArity {
        name: String,
        expected: usize,
        got: usize,
    },
    GeneratedSymbolSpaceExhausted,
}

impl fmt::Display for FrontendIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingIndexedOverload { name } => formatter.write_str(
                &Error::new(format!("missing indexed overload for `{name}`")).render_detail(),
            ),
            Self::MonomorphisationQueueChanged => formatter.write_str(
                &Error::new("monomorphisation queue changed while checking an item")
                    .render_detail(),
            ),
            Self::UnknownGenericStruct { name } => formatter
                .write_str(&Error::new(format!("Unknown generic struct {name}")).render_detail()),
            Self::MissingGenericParameters { name } => formatter.write_str(
                &Error::new(format!("Struct {name} has no generic params")).render_detail(),
            ),
            Self::GenericStructArity {
                name,
                expected,
                got,
            } => formatter.write_str(
                &Error::new(format!(
                    "struct '{name}' expects at most {expected} generic arguments, got {got}"
                ))
                .render_detail(),
            ),
            Self::GeneratedSymbolSpaceExhausted => {
                formatter.write_str(&Error::new("generated symbol space exhausted").render_detail())
            }
        }
    }
}

pub fn missing_standard_library_module(module: &str, shipped: &[&str]) -> CompilationFailure {
    CompilationFailure::from_error(
        Error::new(format!(
            "standard library module `{module}` is not part of X 1.0; shipped modules: {}",
            shipped.join(", ")
        ))
        .with_code("E0015"),
    )
}

pub fn standard_library_parse_failure(
    module: &str,
    source: &str,
    failure: CompilationFailure,
) -> CompilationFailure {
    failure.with_source(source, module).with_note(format!(
        "syntax error inside shipped standard library module `{module}`"
    ))
}

pub fn source_error_label(span: Span) -> Label {
    Label::primary(span, "error occurs here")
}
