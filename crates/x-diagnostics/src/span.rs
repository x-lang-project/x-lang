//! Source identities and half-open byte ranges.
mod source;

pub use source::{FileDatabase, FileRecord};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct SourceIdentifier(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub source: SourceIdentifier,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn single(source: SourceIdentifier, position: usize) -> Self {
        Self {
            source,
            start: position,
            end: position.saturating_add(1),
        }
    }

    pub fn join(first_value: Span, second_value: Span) -> Option<Self> {
        if first_value.source != second_value.source {
            return None;
        }

        Some(Self {
            source: first_value.source,
            start: first_value.start.min(second_value.start),
            end: first_value.end.max(second_value.end),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }
}
