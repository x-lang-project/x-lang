use std::fs;

use std::collections::BTreeSet;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::{Config, Report, Source, scope::Scope};

pub(crate) fn layout(rule: &str) -> bool {
    rule == "unformatted-file"
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckState {
    Completed,
    Disabled,
    Failed,
}

#[derive(Debug, Serialize)]
pub struct FormattingCheck {
    pub rule: &'static str,
    pub status: CheckState,
    pub detail: String,
}

pub(crate) fn checks(config: &Config) -> Vec<FormattingCheck> {
    let rule = "unformatted-file";
    let (status, detail) = if !config.check_format || !crate::profile::enabled(config, rule) {
        (
            CheckState::Disabled,
            "Disabled by configuration or rule selection",
        )
    } else {
        (CheckState::Completed, "Formatting comparison completed")
    };

    vec![FormattingCheck {
        rule,
        status,
        detail: detail.into(),
    }]
}

#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub source_root: String,
    pub completed_unix_seconds: u64,
    pub changed_since_read: Vec<String>,
}

pub(crate) fn snapshot(root: &Path, sources: &[Source]) -> Snapshot {
    Snapshot {
        source_root: root.to_string_lossy().into_owned(),
        completed_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        changed_since_read: sources
            .iter()
            .filter(|source| {
                fs::read_to_string(root.join(&source.path))
                    .map_or(true, |current| current != source.text)
            })
            .map(|source| source.path.clone())
            .collect(),
    }
}

pub fn render(report: &Report) -> String {
    let mut text = format!(
        "Rust review: {} files, {} rules ({} selected), {} candidates, {} errors\n",
        report.files_scanned,
        report.rules_available,
        report.rules_selected,
        report.findings.len(),
        report.errors.len(),
    );
    text.push_str("Review candidates before changing code.\n");
    text.push_str("Formatting is counted once per file, at its first differing line.\n");
    text.push_str(&format!(
        "Profile: {:?}; root: {}\n",
        report.config.profile, report.snapshot.source_root
    ));
    for check in &report.formatting_checks {
        text.push_str(&format!(
            "{}: {:?} — {}\n",
            check.rule, check.status, check.detail
        ));
    }

    if !report.snapshot.changed_since_read.is_empty() {
        text.push_str(&format!(
            "WARNING: {} files changed during the scan; findings use the earlier snapshot.\n",
            report.snapshot.changed_since_read.len()
        ));

        for path in &report.snapshot.changed_since_read {
            text.push_str(&format!("CHANGED {path}\n"));
        }
    }

    for (scope, title, summary) in [
        (Scope::Source, "Regular source", &report.source_summary),
        (Scope::Test, "Tests", &report.test_summary),
    ] {
        text.push_str(&format!(
            "\n{title}: {} candidates in {} affected files\n",
            summary.findings, summary.affected_files
        ));
        text.push_str(&format!(
            "  Formatting differences: {}; review candidates: {}\n",
            summary.formatting_candidates, summary.review_candidates
        ));
        for (rule, count) in &summary.by_rule {
            text.push_str(&format!("  {rule}: {count}\n"));
        }

        for (formatting, label) in [
            (false, "Review candidates"),
            (true, "Formatting differences"),
        ] {
            let rows: Vec<_> = report
                .findings
                .iter()
                .filter(|function| function.scope == scope && layout(&function.rule) == formatting)
                .collect();

            if rows.is_empty() {
                continue;
            }

            text.push_str(&format!("\n{label}\n"));

            for finding in rows {
                text.push_str(&format!(
                    "{}:{}:{} [{}] {}\n  {}\n",
                    finding.path,
                    finding.line,
                    finding.column,
                    finding.rule,
                    finding.message,
                    finding.evidence
                ));
            }
        }
    }

    text.push_str(&format!(
        "\n{} candidates excluded by profile, disabled rules or explicit suppressions.\n",
        report.suppressed.len()
    ));
    for error in &report.errors {
        text.push_str(&format!("ERROR {}: {}\n", error.path, error.message));
    }

    text
}

pub(crate) fn affected_files(findings: &[crate::Finding], scope: Scope) -> usize {
    findings
        .iter()
        .filter(|function| function.scope == scope)
        .map(|function| &function.path)
        .collect::<BTreeSet<_>>()
        .len()
}
