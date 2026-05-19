use std::fmt;

use super::{CompilationFailure, FailureKind};
use crate::{Diagnostic, render_compiler_diagnostic};

pub(super) fn render_fallback_message(
    diagnostic: &Diagnostic,
    code: &str,
    kind: FailureKind,
    stage: Option<&str>,
) -> String {
    let mut message = if kind == FailureKind::Internal {
        crate::render_internal_compiler_error(
            code,
            stage.unwrap_or("compiler"),
            &diagnostic.message,
        )
    } else {
        diagnostic.message.clone()
    };
    // Structured metadata stays unrendered for later source attachment.
    for note in &diagnostic.notes {
        message.push_str(&format!("\nnote: {note}"));
    }

    if let Some(help) = &diagnostic.help {
        message.push_str(&format!("\nhelp: {help}"));
    }

    message
}

impl fmt::Display for CompilationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.internal_stage.is_none()
            && let Some((path, source)) = &self.0.source
        {
            return write!(
                formatter,
                "{}",
                render_compiler_diagnostic(source, path, self.0.diagnostic.clone())
            );
        }

        let message = render_fallback_message(
            &self.0.diagnostic,
            &self.0.code,
            self.0.kind,
            self.0.internal_stage.as_deref(),
        );

        if self.0.kind == FailureKind::Source {
            write!(formatter, "error[{}]: {message}", self.0.code)
        } else {
            formatter.write_str(&message)
        }
    }
}
