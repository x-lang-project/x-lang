use std::fmt::{Debug, Display};

use crate::{Diagnostic, Error, Label, Span};

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

pub fn mir_unsupported_expression(detail: impl Into<String>, span: Span) -> Diagnostic {
    Error::new("unsupported MIR expression")
        .with_code("M0006")
        .with_label(Label::primary(span, detail))
        .with_help("rewrite the expression as simpler statements")
}

pub fn mir_type_promotion_mismatch_display(from: impl Debug, to: impl Debug) -> String {
    Error::new(format!("cannot promote or cast {from:?} to {to:?}")).render_detail()
}

pub fn mir_missing_active_block_context_display() -> String {
    Error::new("no active MIR block").render_detail()
}

pub fn mir_unsupported_expression_display(detail: impl Display) -> String {
    Error::new(format!("Unsupported MIR expression: {detail}")).render_detail()
}
