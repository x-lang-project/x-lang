use std::fs;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{Config, Report, ScanError, Source, analyze, rules};
use x_diagnostics::tooling_messages::SourceReviewError;

fn compile_pattern(pattern: &str) -> Option<regex::Regex> {
    let mut regex = String::from("^");

    for part in pattern.chars() {
        match part {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            character => regex.push_str(&regex::escape(&character.to_string())),
        }
    }

    regex.push('$');

    regex::Regex::new(&regex).ok()
}

struct Exclusions<'a> {
    patterns: Vec<(&'a str, Option<regex::Regex>)>,
}
impl<'a> Exclusions<'a> {
    fn new(config: &'a Config) -> Self {
        Self {
            patterns: config
                .exclude
                .iter()
                .map(|pattern| (pattern.as_str(), compile_pattern(pattern)))
                .collect(),
        }
    }

    fn contains(&self, name: &str) -> bool {
        self.patterns.iter().any(|(pattern, compiled)| {
            name.split('/').any(|part| part == *pattern)
                || compiled
                    .as_ref()
                    .is_some_and(|compiled| compiled.is_match(name))
        })
    }
}

pub(crate) fn paths(root: &Path, config: &Config) -> Result<Vec<PathBuf>, String> {
    inventory(root, config).map(|(paths, _)| paths)
}

fn inventory(root: &Path, config: &Config) -> Result<(Vec<PathBuf>, &'static str), String> {
    let exclusions = Exclusions::new(config);
    let git = Command::new("git")
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ])
        .current_dir(root)
        .output();
    let mut names = BTreeSet::new();
    let method;

    if let Ok(output) = git
        && output.status.success()
    {
        method = "git tracked and nonignored untracked files";
        let text = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;

        for name in text.split('\0').filter(|name| !name.is_empty()) {
            if !exclusions.contains(name) {
                names.insert(root.join(name));
            }
        }
    } else {
        method = "filesystem walk; gitignore unavailable";

        for entry in walkdir::WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| !exclusions.contains(&relative(root, entry.path())))
        {
            let entry = entry.map_err(|error| error.to_string())?;

            if entry.file_type().is_file() || entry.file_type().is_symlink() {
                names.insert(entry.into_path());
            }
        }
    }

    Ok((
        names
            .into_iter()
            .filter(|path| path.extension().is_some_and(|ext| ext == "rs") && path.exists())
            .collect(),
        method,
    ))
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(crate) fn scan(root: &Path, config: &Config) -> Result<Report, String> {
    let root = root.canonicalize().map_err(|error| error.to_string())?;

    if !root.is_dir() {
        return Err(SourceReviewError::root_not_directory());
    }

    if config.maximum_file_bytes == 0 {
        return Err(SourceReviewError::invalid_file_limit());
    }

    let available = rules();
    let identifiers: BTreeSet<_> = available.iter().map(|rule| rule.identifier).collect();

    for identifier in config.disabled_rules.iter().chain(&config.only_rules) {
        if !identifiers.contains(identifier.as_str()) {
            return Err(SourceReviewError::unknown_rule(identifier));
        }
    }

    let mut sources = vec![];
    let mut errors = vec![];
    let (paths, inventory_method) = inventory(&root, config)?;

    for path in paths {
        let name = relative(&root, &path);
        let read = || -> Result<Source, String> {
            let metadata = path.symlink_metadata().map_err(|error| error.to_string())?;

            if metadata.file_type().is_symlink()
                || !path
                    .canonicalize()
                    .map_err(|failure| failure.to_string())?
                    .starts_with(&root)
            {
                return Err(SourceReviewError::source_outside_root());
            }

            if metadata.len() > config.maximum_file_bytes as u64 {
                return Err(SourceReviewError::file_limit_exceeded());
            }

            let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;

            Source::parse(&name, text).map_err(|error| error.to_string())
        };

        match read() {
            Ok(source) => sources.push(source),
            Err(message) => errors.push(ScanError {
                path: name,
                message,
            }),
        }
    }

    crate::module_scopes::classify(&mut sources);
    let mut findings = vec![];
    let mut suppressed = vec![];
    let mut detected = analyze(&sources);
    let mut formatting_checks = crate::reporting::checks(config);

    if config.check_format && crate::profile::enabled(config, "unformatted-file") {
        for source in &sources {
            if let Err(message) = crate::formatter::check(source, &mut detected) {
                formatting_checks[0].status = crate::reporting::CheckState::Failed;
                formatting_checks[0].detail =
                    "At least one rustfmt comparison failed; see scan errors".into();
                errors.push(ScanError {
                    path: source.path.clone(),
                    message,
                });
            }
        }
    }

    detected.sort_by(|first_value, second_value| {
        (
            &first_value.path,
            first_value.line,
            first_value.column,
            &first_value.rule,
        )
            .cmp(&(
                &second_value.path,
                second_value.line,
                second_value.column,
                &second_value.rule,
            ))
    });
    let suppressions: Vec<_> = config
        .suppressions
        .iter()
        .map(|current_text| {
            (
                compile_pattern(&current_text.path),
                compile_pattern(&current_text.rule),
            )
        })
        .collect();

    for finding in detected {
        let disabled = !crate::profile::enabled(config, &finding.rule)
            || suppressions.iter().any(|(path, rule)| {
                path.as_ref()
                    .is_some_and(|path| path.is_match(&finding.path))
                    && rule
                        .as_ref()
                        .is_some_and(|rule| rule.is_match(&finding.rule))
            });

        if disabled {
            suppressed.push(finding);
        } else {
            findings.push(finding);
        }
    }

    let mut by_rule = BTreeMap::new();

    for finding in &findings {
        *by_rule.entry(finding.rule.clone()).or_insert(0) += 1;
    }

    Ok(Report {
        schema_version: 3,
        read_only: true,
        files_scanned: sources.len(),
        rules_available: available.len(),
        rules_selected: available.iter().filter(|rule| crate::profile::enabled(config, rule.identifier)).count(),
        by_rule,
        source_summary: crate::scope::ScopeSummary::from_findings(&findings, crate::scope::Scope::Source),
        test_summary: crate::scope::ScopeSummary::from_findings(&findings, crate::scope::Scope::Test),
        findings,
        suppressed,
        errors,
        inventory_method: inventory_method.into(),
        config: config.clone(),
        formatting_checks,
        snapshot: crate::reporting::snapshot(&root, &sources),
        analysis_limits: vec!["Syntax review without type resolution or macro expansion.".into(),
            "Formatting is reported per file at its first differing line.".into(),
            "Default trait methods are included.".into(),
            "Module lookup uses src paths and inline modules. Custom targets, #[path], include!, cfg and import aliases are unresolved. Files outside src have separate roots; shared library/binary roots are not distinguished.".into(),
            "Near-duplicate search uses token sets of at least 24 whitespace-delimited tokens and five-token shingles; buckets above 64 functions are skipped.".into(),
            "Rustfmt uses edition 2024 and Unix line endings; import/module order is preserved and syntax is checked.".into()],
    })
}
