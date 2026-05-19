use serde::Serialize;

use crate::{Diagnostic, FileDatabase, LabelStyle};

#[derive(Debug, Serialize)]
pub struct SerializableDiagnostic {
    pub kind: String,
    pub severity: String,
    #[serde(rename = "primary_span")]
    pub span: Option<SerializableSpan>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub repair_policies: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub message: String,
}
#[derive(Debug, Serialize)]
pub struct SerializableSpan {
    pub file: String,
    pub start: [usize; 2],
    pub end: [usize; 2],
}

pub(crate) fn serializable_diagnostic(
    database: &FileDatabase,
    diagnostic: &Diagnostic,
) -> SerializableDiagnostic {
    let payload = diagnostic.json_payload.as_ref();

    SerializableDiagnostic {
        kind: payload
            .map(|payload| payload.kind.clone())
            .or_else(|| diagnostic.code.clone())
            .unwrap_or_else(|| message_kind(&diagnostic.message)),
        severity: diagnostic.severity.name().to_string(),
        span: primary_span(database, diagnostic),
        repair_policies: payload
            .map(|payload| payload.repair_policies.clone())
            .unwrap_or_default(),
        caller: payload.and_then(|payload| payload.caller.clone()),
        callee: payload.and_then(|payload| payload.callee.clone()),
        code: diagnostic.code.clone(),
        message: diagnostic.message.clone(),
    }
}

fn message_kind(message: &str) -> String {
    let mut output = String::new();
    let mut uppercase_next = true;

    for ch in message.chars() {
        if ch.is_ascii_alphanumeric() {
            if uppercase_next {
                output.push(ch.to_ascii_uppercase());
                uppercase_next = false;
            } else {
                output.push(ch);
            }
        } else {
            uppercase_next = true;
        }
    }

    if output.is_empty() {
        "Diagnostic".to_string()
    } else {
        output
    }
}

fn primary_span(database: &FileDatabase, diagnostic: &Diagnostic) -> Option<SerializableSpan> {
    diagnostic
        .labels
        .iter()
        .find(|label| label.style == LabelStyle::Primary)
        .or_else(|| diagnostic.labels.first())
        .and_then(|label| {
            let file = database.get(label.span.source)?;
            let (start_line, start_column) = file.line_column(label.span.start);
            let end_byte = label.span.end.saturating_sub(1).max(label.span.start);
            let (end_line, end_column) = file.line_column(end_byte);

            Some(SerializableSpan {
                file: file.path.display().to_string(),
                start: [start_line, start_column],
                end: [end_line, end_column],
            })
        })
}
