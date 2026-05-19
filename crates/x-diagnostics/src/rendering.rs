use std::fmt::Display;
use std::path::Path;

use crate::source_lookup::enrich_missing_spans;
use crate::{
    DEFAULT_DIAGNOSTIC_WIDTH, Diagnostic, DiagnosticFact, DiagnosticFix, DiagnosticStyle, Emitter,
    Error, FactKind, FileDatabase, Label, SourceIdentifier, Span, StageDiagnostic,
    pretty_diagnostics_enabled,
};

pub fn source_span_for_diagnostic(source_identifier: SourceIdentifier, source: &str) -> Span {
    let start = source
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(index, _)| index)
        .unwrap_or(0);
    let line_end = source[start..]
        .find('\n')
        .map(|offset| start + offset)
        .unwrap_or_else(|| source.len());
    let end = line_end.max(start + 1).min(source.len().max(1));

    Span {
        source: source_identifier,
        start,
        end,
    }
}

pub fn stage_error_diagnostic(
    stage: impl Into<String>,
    code: impl Into<String>,
    error: impl ToString,
    span: Span,
) -> Diagnostic {
    let stage = stage.into();

    Error::new(format!("{stage} failed"))
        .with_code(code)
        .with_label(Label::primary(span, error.to_string()))
        .with_fact(
            DiagnosticFact::new(FactKind::Found, format!("reported during {stage}")).at(span),
        )
        .with_fix(DiagnosticFix::high(
            "fix the highlighted source and run the command again",
        ))
        .with_fix(DiagnosticFix::medium(
            "check the line before the highlight for a missing token or delimiter",
        ))
        .with_help("fix the highlighted source and run the command again")
}

pub fn render_compiler_diagnostic(
    source: &str,
    path: impl AsRef<Path>,
    diagnostic: Diagnostic,
) -> String {
    let mut database = FileDatabase::default();
    let source_identifier = database.add_file(path.as_ref(), source.to_string());
    let diagnostic = enrich_missing_spans(source_identifier, source, diagnostic);

    render_diagnostic(&database, diagnostic)
}

pub fn render_internal_compiler_error(code: &str, stage: &str, detail: impl Display) -> String {
    let code = if code.starts_with("ICE-") {
        code.to_string()
    } else {
        format!("ICE-{code}")
    };

    Error::new(format!("internal compiler error during {stage}"))
        .with_code(&code)
        .with_note(detail.to_string())
        .with_note(format!(
            "reduce the input while preserving diagnostic {code}"
        ))
        .with_note("report the reduced source, command, `x --version` and diagnostic ID")
        .render_plain()
}

pub fn render_stage_error(
    source: &str,
    path: impl AsRef<Path>,
    identity: StageDiagnostic<'_>,
    error: impl ToString,
) -> String {
    let mut database = FileDatabase::default();
    let source_identifier = database.add_file(path.as_ref(), source.to_string());
    let span = source_span_for_diagnostic(source_identifier, source);
    let diagnostic = stage_error_diagnostic(identity.stage, identity.code, error, span);

    render_diagnostic(&database, diagnostic)
}

fn render_diagnostic(database: &FileDatabase, diagnostic: Diagnostic) -> String {
    let mut output = Vec::new();
    let emitter = diagnostic_emitter_for_current_settings();

    if emitter.emit(database, &diagnostic, &mut output).is_err() {
        return diagnostic.message;
    }

    String::from_utf8_lossy(&output).trim_end().to_string()
}

fn diagnostic_emitter_for_current_settings() -> Emitter {
    if pretty_diagnostics_enabled() {
        Emitter::default()
    } else {
        Emitter {
            use_colour: false,
            tty_width: DEFAULT_DIAGNOSTIC_WIDTH,
            style: DiagnosticStyle::Compact,
        }
    }
}
