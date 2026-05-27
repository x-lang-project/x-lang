mod facts;
mod label;
mod payload;
mod suggestion;

use crate::CompilationFailure;

pub use crate::serialise::{SerializableDiagnostic, SerializableSpan};
pub use facts::{DiagnosticFact, DiagnosticFix, FactKind, FixConfidence};
pub use label::{Label, LabelStyle};
pub use payload::JsonDiagnosticPayload;
pub use suggestion::{Applicability, SpanEdit, Suggestion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

impl Severity {
    pub fn name(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Note => "note",
            Self::Help => "help",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub help: Option<String>,
    pub suggestions: Vec<Suggestion>,
    pub facts: Vec<DiagnosticFact>,
    pub fixes: Vec<DiagnosticFix>,
    pub json_payload: Option<JsonDiagnosticPayload>,
}

pub type Error = Diagnostic;

impl Diagnostic {
    fn with_severity(severity: Severity, current_message: impl Into<String>) -> Self {
        Self {
            severity,
            code: None,
            message: current_message.into(),
            labels: vec![],
            notes: vec![],
            help: None,
            suggestions: vec![],
            facts: vec![],
            fixes: vec![],
            json_payload: None,
        }
    }

    pub fn new(message: impl Into<String>) -> Self {
        Self::with_severity(Severity::Error, message)
    }

    pub fn into_failure(self) -> CompilationFailure {
        CompilationFailure::from_error(self)
    }

    pub fn into_internal_failure(self, stage: impl Into<String>) -> CompilationFailure {
        CompilationFailure::from_internal_error(self, stage.into())
    }

    pub fn render_plain(&self) -> String {
        let heading = match &self.code {
            Some(code) => format!("error[{code}]: {}", self.message),
            None => self.message.clone(),
        };

        self.render_metadata(heading)
    }

    pub fn render_detail(&self) -> String {
        let heading = match &self.code {
            Some(code) => format!("{code}: {}", self.message),
            None => self.message.clone(),
        };

        self.render_metadata(heading)
    }

    fn render_metadata(&self, mut message: String) -> String {
        for note in &self.notes {
            message.push_str(&format!("\nnote: {note}"));
        }

        if let Some(help) = &self.help {
            message.push_str(&format!("\nhelp: {help}"));
        }

        message
    }

    pub fn note(message: impl Into<String>) -> Self {
        Self::with_severity(Severity::Note, message)
    }

    pub fn help_diag(message: impl Into<String>) -> Self {
        Self::with_severity(Severity::Help, message)
    }

    pub fn with_code(mut self, character: impl Into<String>) -> Self {
        self.code = Some(character.into());
        self
    }

    pub fn with_label(mut self, left_value: Label) -> Self {
        self.labels.push(left_value);
        self
    }

    pub fn with_help(mut self, h: impl Into<String>) -> Self {
        self.help = Some(h.into());
        self
    }

    pub fn with_note(mut self, count: impl Into<String>) -> Self {
        self.notes.push(count.into());
        self
    }

    pub fn with_suggestion(mut self, text: Suggestion) -> Self {
        self.suggestions.push(text);
        self
    }

    pub fn with_fact(mut self, fact: DiagnosticFact) -> Self {
        self.facts.push(fact);
        self
    }

    pub fn with_fix(mut self, fix: DiagnosticFix) -> Self {
        self.fixes.push(fix);
        self
    }

    pub fn with_json_payload(mut self, payload: JsonDiagnosticPayload) -> Self {
        self.json_payload = Some(payload);
        self
    }

    pub fn is_error(&self) -> bool {
        matches!(self.severity, Severity::Error)
    }
}
