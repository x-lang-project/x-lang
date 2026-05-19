use std::fmt::{Debug, Display};

use crate::{Diagnostic, Error, Label, Span};

fn internal_codegen_diagnostic(
    message: impl Into<String>,
    code: &'static str,
    span: Span,
    detail: impl Into<String>,
) -> Diagnostic {
    Error::new(message)
        .with_code(code)
        .with_label(Label::primary(span, detail))
        .with_note(format!(
            "reduce the input while preserving diagnostic {code}"
        ))
        .with_note(format!(
            "report the reduced source, command, `x --version` and diagnostic {code}"
        ))
        .with_help("the compiler failed an internal invariant")
}

mod display;
pub use display::*;
