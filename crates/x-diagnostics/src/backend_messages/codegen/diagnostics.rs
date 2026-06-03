use std::fmt::{Debug, Display};

use crate::{Diagnostic, Error, Label, Span};

pub fn codegen_llvm_verification_failure(detail: impl Into<String>, span: Span) -> Diagnostic {
    internal_codegen_diagnostic(
        "generated LLVM IR failed verification",
        "ICE-C0020",
        span,
        detail,
    )
}

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
