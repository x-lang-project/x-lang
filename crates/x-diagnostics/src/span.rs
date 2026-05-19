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
