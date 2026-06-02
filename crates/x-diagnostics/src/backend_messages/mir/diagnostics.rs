use std::fmt::{Debug, Display};

use crate::{Diagnostic, Error, Label, Span};

pub fn mir_unallocated_variable(name: impl Display, span: Span) -> Diagnostic {
    Error::new("MIR lowering failed")
        .with_code("M0001")
        .with_label(Label::primary(
            span,
            format!("`{name}` has no MIR local allocation"),
        ))
        .with_help("ensure the value is declared before it is lowered")
}

pub fn mir_invalid_memory_place(place: impl Into<String>, span: Span) -> Diagnostic {
    Error::new("invalid MIR memory place")
        .with_code("M0002")
        .with_label(Label::primary(span, place))
        .with_help("simplify the expression or lower it into an explicit temporary")
}

pub fn mir_type_promotion_mismatch(from: impl Debug, to: impl Debug, span: Span) -> Diagnostic {
    Error::new("MIR type promotion failed")
        .with_code("M0003")
        .with_label(Label::primary(
            span,
            format!("cannot promote `{from:?}` to `{to:?}`"),
        ))
        .with_help("add an explicit cast before MIR lowering")
}

pub fn mir_missing_active_block_context(span: Span) -> Diagnostic {
    Error::new("missing active MIR block")
        .with_code("M0004")
        .with_label(Label::primary(
            span,
            "lowering attempted to emit without an active block",
        ))
        .with_help("this is a compiler bug; reduce the input and report it")
}

pub fn mir_invalid_static_initializer(detail: impl Into<String>, span: Span) -> Diagnostic {
    Error::new("invalid static initialiser")
        .with_code("M0005")
        .with_label(Label::primary(span, detail))
        .with_help("use a compile-time constant initializer")
}

pub fn mir_unsupported_expression(detail: impl Into<String>, span: Span) -> Diagnostic {
    Error::new("unsupported MIR expression")
        .with_code("M0006")
        .with_label(Label::primary(span, detail))
        .with_help("rewrite the expression as simpler statements")
}

pub fn mir_invalid_contract(detail: impl Into<String>, span: Span) -> Diagnostic {
    Error::new("invalid MIR contract")
        .with_code("M0007")
        .with_label(Label::primary(span, detail))
        .with_help("contract primitives must remain typed and canonical through MIR")
}

pub fn mir_unallocated_variable_display(name: impl Display) -> String {
    Error::new(format!("MIR Lowering Lookup Fault: Variable or symbol allocation '{name}' was never declared in this context frame.")).render_detail()
}

pub fn mir_invalid_memory_place_display(place: impl Display) -> String {
    Error::new(format!("invalid MIR place: {place}")).render_detail()
}

pub fn mir_type_promotion_mismatch_display(from: impl Debug, to: impl Debug) -> String {
    Error::new(format!("cannot promote or cast {from:?} to {to:?}")).render_detail()
}

pub fn mir_missing_active_block_context_display() -> String {
    Error::new("no active MIR block").render_detail()
}

pub fn mir_invalid_static_initializer_display() -> String {
    Error::new("Invalid Static Initialiser").render_detail()
}

pub fn mir_unsupported_expression_display(detail: impl Display) -> String {
    Error::new(format!("Unsupported MIR expression: {detail}")).render_detail()
}

pub fn mir_invalid_contract_display(detail: impl Display) -> String {
    Error::new(format!("Invalid semantic contract program: {detail}")).render_detail()
}
