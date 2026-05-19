use crate::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applicability {
    MachineApplicable,
    MaybeIncorrect,
    HasPlaceholders,
    Unspecified,
}

#[derive(Debug, Clone)]
pub struct SpanEdit {
    pub span: Span,
    pub replacement: String,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub applicability: Applicability,
    pub edits: Vec<SpanEdit>,
}
