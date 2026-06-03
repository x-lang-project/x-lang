use std::fmt::{Debug, Display};

use crate::{Diagnostic, Error, Label, Span};

pub fn codegen_builder_error(error: impl Debug, span: Span) -> Diagnostic {
    internal_codegen_diagnostic(
        "LLVM builder failed",
        "ICE-C0001",
        span,
        format!("{error:?}"),
    )
}

pub fn codegen_type_not_found(name: impl Display, span: Span) -> Diagnostic {
    internal_codegen_diagnostic(
        "validated type has no registered LLVM layout",
        "ICE-C0002",
        span,
        format!("`{name}` was not registered in this module"),
    )
}

pub fn codegen_unsupported_type(value_type: impl Debug, span: Span) -> Diagnostic {
    Error::new("unsupported backend type")
        .with_code("C0003")
        .with_label(Label::primary(
            span,
            format!("`{value_type:?}` cannot be represented in LLVM"),
        ))
        .with_help("lower this type before code generation or add backend support")
}

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

pub fn codegen_missing_local_allocation(identifier: impl Display, span: Span) -> Diagnostic {
    internal_codegen_diagnostic(
        "validated MIR local has no backend allocation",
        "ICE-C0007",
        span,
        format!("local `{identifier}` was used before stack allocation"),
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

pub fn codegen_type_has_no_size(value_type: impl Debug, span: Span) -> Diagnostic {
    Error::new("type has no backend size")
        .with_code("C0012")
        .with_label(Label::primary(
            span,
            format!("`{value_type:?}` has no known size"),
        ))
        .with_help("use a sized type or define its layout before code generation")
}

pub fn codegen_missing_callee_symbol(name: impl Display, span: Span) -> Diagnostic {
    internal_codegen_diagnostic(
        "validated call has no backend callee symbol",
        "ICE-C0013",
        span,
        format!("`{name}` was not resolved"),
    )
}

pub fn codegen_jit_execution_failure(name: impl Display, span: Span) -> Diagnostic {
    Error::new("JIT execution failed")
        .with_code("C0015")
        .with_label(Label::primary(span, format!("could not execute `{name}`")))
        .with_help("check that the entry function exists and has a supported signature")
}

pub fn codegen_llvm_internal_error(detail: impl Into<String>, span: Span) -> Diagnostic {
    internal_codegen_diagnostic("LLVM backend invariant failed", "ICE-C0016", span, detail)
}

pub fn codegen_jit_initialization_failure(detail: impl Into<String>, span: Span) -> Diagnostic {
    Error::new("LLVM JIT initialisation failed")
        .with_code("C0017")
        .with_label(Label::primary(span, detail))
        .with_help("run `x doctor` to verify the installed LLVM toolchain")
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

pub fn codegen_invalid_mir(detail: impl Into<String>, span: Span) -> Diagnostic {
    internal_codegen_diagnostic(
        "invalid MIR reached code generation",
        "ICE-C0021",
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
