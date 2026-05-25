use std::error;
use std::path::{Path, PathBuf};

use crate::Label;
use crate::Span;
use crate::cli;
use crate::{Diagnostic, SourceIdentifier, source_span_for_diagnostic, stage_error_diagnostic};

mod identity;
mod presentation;

pub use identity::{FailureKind, StageDiagnostic};

#[derive(Debug, Clone)]
pub struct CompilationFailure(Box<FailureData>);
#[derive(Debug, Clone)]
struct FailureData {
    kind: FailureKind,
    code: String,
    diagnostic: Diagnostic,
    source: Option<(PathBuf, String)>,
    span: Option<(usize, usize)>,
    internal_stage: Option<String>,
}
impl FailureData {
    fn from_diagnostic(mut diagnostic: Diagnostic) -> Self {
        let code = diagnostic
            .code
            .get_or_insert_with(|| cli::OPERATIONAL_CODE.into())
            .clone();

        Self {
            kind: FailureKind::from_code(&code),
            code,
            diagnostic,
            source: None,
            span: None,
            internal_stage: None,
        }
    }
}

impl CompilationFailure {
    pub fn from_error(diagnostic: Diagnostic) -> Self {
        Self(Box::new(FailureData::from_diagnostic(diagnostic)))
    }

    pub fn diagnostic(source: &str, path: impl AsRef<Path>, diagnostic: Diagnostic) -> Self {
        let mut data = FailureData::from_diagnostic(diagnostic);
        data.source = Some((path.as_ref().into(), source.into()));

        Self(Box::new(data))
    }

    pub(crate) fn from_internal_error(diagnostic: Diagnostic, stage: String) -> Self {
        let mut failure = Self::from_error(diagnostic);
        failure.0.kind = FailureKind::Internal;
        failure.0.internal_stage = Some(stage);

        failure
    }

    pub fn code(&self) -> &str {
        &self.0.code
    }

    pub fn as_diagnostic(&self) -> &Diagnostic {
        &self.0.diagnostic
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.0.diagnostic.notes.push(note.into());
        self
    }

    pub fn with_span(mut self, start: usize, end: usize) -> Self {
        self.0.span = Some((start, end));
        self
    }

    pub fn with_source_if_missing(self, source: &str, path: impl AsRef<Path>) -> Self {
        if self.0.source.is_some() {
            self
        } else {
            self.with_source(source, path)
        }
    }

    pub fn with_source(mut self, source: &str, path: impl AsRef<Path>) -> Self {
        if self.0.kind == FailureKind::Source && self.0.diagnostic.labels.is_empty() {
            let span = self
                .0
                .span
                .map(|(start, end)| Span {
                    source: SourceIdentifier(0),
                    start: start.min(source.len()),
                    end: end.min(source.len()).max(start.min(source.len())),
                })
                .unwrap_or_else(|| source_span_for_diagnostic(SourceIdentifier(0), source));

            self.0
                .diagnostic
                .labels
                .push(Label::primary(span, "source error"));
        }
        self.0.source = Some((path.as_ref().into(), source.into()));
        self
    }

    pub fn contains(&self, text: &str) -> bool {
        self.to_string().contains(text)
    }
}

impl From<Diagnostic> for CompilationFailure {
    fn from(diagnostic: Diagnostic) -> Self {
        Self::from_error(diagnostic)
    }
}

impl error::Error for CompilationFailure {}
