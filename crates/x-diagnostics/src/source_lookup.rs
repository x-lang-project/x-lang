mod matching;
mod quotes;

use crate::{Diagnostic, Label, SourceIdentifier, Span};
use matching::{find_identifier, find_mutating_assignment};
use quotes::{
    backtick_values, first_backtick_before, is_source_lookup_candidate, second_backtick_between,
};

pub(super) fn enrich_missing_spans(
    source_identifier: SourceIdentifier,
    source: &str,
    mut diagnostic: Diagnostic,
) -> Diagnostic {
    if !diagnostic.labels.is_empty() {
        return diagnostic;
    }

    if diagnostic.code.as_deref() == Some("E0006")
        && let Some(name) = unresolved_name_from_diagnostic(&diagnostic)
        && let Some(start) = find_identifier(source, &name)
    {
        let span = Span {
            source: source_identifier,
            start,
            end: start + name.len(),
        };
        diagnostic.labels.push(Label::primary(
            span,
            format!("`{name}` is used here, but no visible declaration named `{name}` was found"),
        ));
    }

    if matches!(diagnostic.code.as_deref(), Some("E0003" | "E0004"))
        && let Some(name) = unresolved_name_from_diagnostic(&diagnostic)
        && let Some(start) = find_identifier(source, &name)
    {
        diagnostic.labels.push(Label::primary(
            Span {
                source: source_identifier,
                start,
                end: start + name.len(),
            },
            format!("`{name}` is the rejected call or member here"),
        ));
    }

    if diagnostic.code.as_deref() == Some("E0002")
        && enrich_purity_span(source_identifier, source, &mut diagnostic)
    {
        return diagnostic;
    }

    if diagnostic.labels.is_empty() && !source.is_empty() {
        diagnostic.labels.push(Label::primary(
            Span {
                source: source_identifier,
                start: 0,
                end: source.len(),
            },
            "this source contains the rejected construct",
        ));
    }

    diagnostic
}

fn enrich_purity_span(
    source_identifier: SourceIdentifier,
    source: &str,
    diagnostic: &mut Diagnostic,
) -> bool {
    if forbidden_pure_assignment(diagnostic)
        && let Some(start) = find_mutating_assignment(source)
    {
        let span = Span {
            source: source_identifier,
            start,
            end: start + 1,
        };
        diagnostic.labels.push(Label::primary(
            span,
            "this assignment can modify state outside the pure function",
        ));

        return true;
    }

    if let Some(name) = purity_operation_from_diagnostic(diagnostic)
        && let Some(start) = find_identifier(source, &name)
    {
        let span = Span {
            source: source_identifier,
            start,
            end: start + name.len(),
        };
        diagnostic.labels.push(Label::primary(
            span,
            format!("`{name}` performs observable operations that are not allowed here"),
        ));
    }

    false
}

pub(super) fn unresolved_name_from_diagnostic(diagnostic: &Diagnostic) -> Option<String> {
    diagnostic
        .notes
        .iter()
        .chain(diagnostic.facts.iter().map(|fact| &fact.message))
        .find_map(|text| {
            let (_, rest) = text.split_once('`')?;
            let (name, _) = rest.split_once('`')?;

            (!name.is_empty()).then(|| name.to_string())
        })
}

fn forbidden_pure_assignment(diagnostic: &Diagnostic) -> bool {
    diagnostic
        .notes
        .iter()
        .any(|note| note.contains("`assignment in pure function`"))
}

fn purity_operation_from_diagnostic(diagnostic: &Diagnostic) -> Option<String> {
    for note in &diagnostic.notes {
        if let Some(operation) = first_backtick_before(note, " is not allowed in ") {
            return Some(operation);
        }

        if let Some(callee) = second_backtick_between(note, " calls ") {
            return Some(callee);
        }
    }

    source_candidate_from_diagnostic(diagnostic)
}

fn source_candidate_from_diagnostic(diagnostic: &Diagnostic) -> Option<String> {
    diagnostic
        .notes
        .iter()
        .chain(diagnostic.facts.iter().map(|fact| &fact.message))
        .flat_map(|text| backtick_values(text))
        .find(|name| is_source_lookup_candidate(name))
        .map(str::to_string)
}
