use crate::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactKind {
    Required,
    Inferred,
    Found,
    Note,
}

#[derive(Debug, Clone)]
pub struct DiagnosticFact {
    pub kind: FactKind,
    pub message: String,
    pub span: Option<Span>,
}

impl DiagnosticFact {
    pub fn new(kind: FactKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            span: None,
        }
    }

    pub fn at(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FixConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct DiagnosticFix {
    pub confidence: FixConfidence,
    pub message: String,
}

impl DiagnosticFix {
    pub fn high(message: impl Into<String>) -> Self {
        Self {
            confidence: FixConfidence::High,
            message: message.into(),
        }
    }

    pub fn medium(message: impl Into<String>) -> Self {
        Self {
            confidence: FixConfidence::Medium,
            message: message.into(),
        }
    }

    pub fn low(message: impl Into<String>) -> Self {
        Self {
            confidence: FixConfidence::Low,
            message: message.into(),
        }
    }
}
