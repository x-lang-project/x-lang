mod baseline;
pub mod clarity;
pub mod efficiency;
mod format_apply;
mod format_pipeline;
pub mod formatter;
mod inventory;
mod masking;
mod module_scopes;

pub mod profile;
pub mod reporting;
mod review_sink;
pub mod scope;
mod scope_includes;
mod source;
use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use review_sink::ReviewSink;

pub mod structure;

#[derive(Clone, Debug, Serialize)]
pub struct Rule {
    pub identifier: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub analysis: &'static str,
    pub positive: &'static str,
    pub negative: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Finding {
    pub scope: scope::Scope,
    pub rule: String,
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub evidence: String,
}

pub struct Source {
    pub path: String,
    pub text: String,
    pub masked: String,
    pub ast: syn::File,
    test_scopes: scope::TestScopes,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub profile: profile::Profile,
    pub check_format: bool,
    pub exclude: Vec<String>,
    pub disabled_rules: Vec<String>,
    pub only_rules: Vec<String>,
    pub suppressions: Vec<Suppression>,
    pub maximum_file_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Suppression {
    pub path: String,
    pub rule: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profile: profile::Profile::Focused,
            check_format: true,

            exclude: [
                "target",
                ".git",
                "backups",
                "work",
                "tmp",
                "output",
                "outputs",
                "research",
                "x-os",
                "vendor",
                "generated",
                "mutants.out",
                "mutants.out.old",
                ".tools",
            ]
            .map(str::to_owned)
            .to_vec(),
            disabled_rules: vec![],
            only_rules: vec![],
            suppressions: vec![],
            maximum_file_bytes: 2_000_000,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ScanError {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub read_only: bool,
    pub files_scanned: usize,
    pub rules_available: usize,
    pub rules_selected: usize,
    pub by_rule: BTreeMap<String, usize>,
    pub source_summary: scope::ScopeSummary,
    pub test_summary: scope::ScopeSummary,
    pub findings: Vec<Finding>,
    pub suppressed: Vec<Finding>,
    pub errors: Vec<ScanError>,
    pub inventory_method: String,
    pub config: Config,
    pub analysis_limits: Vec<String>,
    pub formatting_checks: Vec<reporting::FormattingCheck>,
    pub snapshot: reporting::Snapshot,
}

pub fn rules() -> Vec<Rule> {
    let mut rules = baseline::rules();
    rules.extend(clarity::rules());
    rules.extend(structure::rules());
    rules.extend(efficiency::rules());
    rules.push(Rule { identifier: "unformatted-file", title: "Rustfmt layout drift",
        description: "Source differs from rustfmt's standard indentation, spacing and line wrapping.",
        analysis: "rustfmt with syntax validation",
        positive: "fn example(){let count=1;}", negative: "fn example() {\n    let count = 1;\n}\n" });

    rules.sort_by_key(|rule| rule.identifier);

    rules
}

pub fn analyze(sources: &[Source]) -> Vec<Finding> {
    let mut findings = vec![];

    for source in sources {
        baseline::analyze(source, &mut findings);
        clarity::analyze(source, &mut findings);
        structure::analyze(source, &mut findings);
        efficiency::analyze(source, &mut findings);
    }

    baseline::analyze_project(sources, &mut findings);
    clarity::analyze_project(sources, &mut findings);
    structure::analyze_project(sources, &mut findings);
    efficiency::analyze_project(sources, &mut findings);
    findings.sort_by(|first_value, second_value| {
        (
            &first_value.path,
            first_value.line,
            first_value.column,
            &first_value.rule,
            &first_value.message,
        )
            .cmp(&(
                &second_value.path,
                second_value.line,
                second_value.column,
                &second_value.rule,
                &second_value.message,
            ))
    });
    findings.dedup();

    findings
}

pub fn scan(root: &Path, config: &Config) -> Result<Report, String> {
    inventory::scan(root, config)
}

pub fn apply_format(root: &Path, config: &Config) -> Result<Vec<String>, String> {
    format_apply::apply(root, config)
}
