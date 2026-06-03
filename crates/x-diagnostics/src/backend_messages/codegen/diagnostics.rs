use std::fmt::{Debug, Display};

use crate::{Diagnostic, Error, Label, Span};

pub fn codegen_function_signature_missing(name: impl Display, span: Span) -> Diagnostic {
    internal_codegen_diagnostic(
        "validated call has no backend signature",
        "ICE-C0004",
        span,
        format!("`{name}` has no registered signature"),
    )
}

pub fn codegen_missing_terminator(block: impl Display, span: Span) -> Diagnostic {
    internal_codegen_diagnostic(
        "validated MIR block has no terminator",
        "ICE-C0005",
        span,
        format!("MIR block {block} has no terminator"),
    )
}

pub fn codegen_missing_entry_block(span: Span) -> Diagnostic {
    internal_codegen_diagnostic(
        "validated MIR function has no entry block",
        "ICE-C0006",
        span,
        "MIR function has no block 0",
    )
}

pub fn codegen_missing_hardware_argument(position: impl Display, span: Span) -> Diagnostic {
    internal_codegen_diagnostic(
        "validated hardware call has no backend argument",
        "ICE-C0008",
        span,
        format!("argument {position} is unavailable"),
    )
}

pub fn codegen_unsupported_operator(operator: impl Debug, span: Span) -> Diagnostic {
    Error::new("unsupported operator")
        .with_code("C0009")
        .with_label(Label::primary(
            span,
            format!("`{operator:?}` is not supported for these operand types"),
        ))
        .with_help("add an explicit cast or avoid this operator combination")
}

pub fn codegen_unsupported_unary_operator(operator: impl Debug, span: Span) -> Diagnostic {
    Error::new("unsupported unary operator")
        .with_code("C0010")
        .with_label(Label::primary(
            span,
            format!("`{operator:?}` is not supported here"),
        ))
        .with_help("rewrite the expression or add backend support")
}

pub fn codegen_unsupported_cast(to: impl Debug, span: Span) -> Diagnostic {
    Error::new("unsupported cast")
        .with_code("C0011")
        .with_label(Label::primary(
            span,
            format!("cannot cast to `{to:?}` in the backend"),
        ))
        .with_help("insert a supported intermediate cast")
}

pub fn codegen_missing_callee_symbol(name: impl Display, span: Span) -> Diagnostic {
    internal_codegen_diagnostic(
        "validated call has no backend callee symbol",
        "ICE-C0013",
        span,
        format!("`{name}` was not resolved"),
    )
}

pub fn codegen_missing_mir_block(block: impl Display, span: Span) -> Diagnostic {
    internal_codegen_diagnostic(
        "validated MIR references a missing block",
        "ICE-C0018",
        span,
        format!("MIR block {block} is absent"),
    )
}

pub fn codegen_missing_llvm_block(block: impl Display, span: Span) -> Diagnostic {
    internal_codegen_diagnostic(
        "validated MIR block has no LLVM mapping",
        "ICE-C0019",
        span,
        format!("MIR block {block} has no LLVM block"),
    )
}

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
