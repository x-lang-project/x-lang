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

impl Suggestion {
    pub fn new(current_message: impl Into<String>) -> Self {
        Self {
            message: current_message.into(),
            applicability: Applicability::Unspecified,
            edits: vec![],
        }
    }

    pub fn applicability(mut self, first_value: Applicability) -> Self {
        self.applicability = first_value;
        self
    }

    pub fn edit(mut self, span: Span, replacement: impl Into<String>) -> Self {
        self.edits.push(SpanEdit {
            span,
            replacement: replacement.into(),
        });
        self
    }
}
