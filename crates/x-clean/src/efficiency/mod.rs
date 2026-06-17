use std::sync::OnceLock;

use regex::Regex;

mod ast_checks;
mod locks;
mod precision;
mod rules0;
mod rules1;
mod rules2;
mod rules3;
mod rules4;
pub(super) enum Detector {
    Tokens(&'static str),
    DifferentCollections(&'static str),
    Ast,
}
impl Detector {
    fn regex(&self) -> Option<&'static str> {
        match self {
            Self::Tokens(pattern) | Self::DifferentCollections(pattern) => Some(pattern),
            Self::Ast => None,
        }
    }
}
pub(super) struct Spec {
    identifier: &'static str,
    pattern: Detector,
    title: &'static str,
    description: &'static str,
    positive: &'static str,
    negative: &'static str,
}
type Compiled = (&'static Spec, Regex, Vec<(String, String)>);
fn specs() -> impl Iterator<Item = &'static Spec> {
    [
        rules0::SPECS,
        rules1::SPECS,
        rules2::SPECS,
        rules3::SPECS,
        rules4::SPECS,
    ]
    .into_iter()
    .flatten()
}

pub fn rules() -> Vec<crate::Rule> {
    specs()
        .map(|text| crate::Rule {
            identifier: text.identifier,
            title: text.title,
            description: text.description,
            analysis: "Masked tokens and Rust syntax",
            positive: text.positive,
            negative: text.negative,
        })
        .collect()
}

fn compile(pattern: &str) -> Option<(Regex, Vec<(String, String)>)> {
    let mut result = String::new();
    let mut groups = 0;
    let mut refs = Vec::new();
    let chars: Vec<_> = pattern.chars().collect();
    let mut index = 0;
    let mut class = false;

    while index < chars.len() {
        let ch = chars[index];

        if ch == '\\' && index + 1 < chars.len() {
            let next = chars[index + 1];

            if next.is_ascii_digit() {
                let name = format!("ref{}", refs.len());
                result.push_str(&format!("(?P<{name}>[A-Za-z_]\\w*)"));
                refs.push((format!("group{next}"), name));
            } else {
                result.push(ch);
                result.push(next);
            }
            index += 2;
            continue;
        }

        if ch == '[' {
            class = true;
        }

        if ch == ']' {
            class = false;
        }

        if ch == '(' && !class && chars.get(index + 1) != Some(&'?') {
            groups += 1;
            result.push_str(&format!("(?P<group{groups}>"));
        } else {
            result.push(ch);
        }
        index += 1;
    }

    Regex::new(&result)
        .ok()
        .map(|current_result| (current_result, refs))
}

pub fn analyze(source: &crate::Source, findings: &mut Vec<crate::Finding>) {
    let mut compact = String::new();
    let mut offsets = Vec::new();

    for (offset, ch) in source.masked.char_indices() {
        if !ch.is_whitespace() {
            compact.push(ch);

            for _ in 0..ch.len_utf8() {
                offsets.push(offset);
            }
        }
    }

    static COMPILED: OnceLock<Vec<Compiled>> = OnceLock::new();
    let compiled = COMPILED.get_or_init(|| {
        specs()
            .filter_map(|spec| {
                let pattern = spec.pattern.regex()?;
                let (regex, refs) = compile(pattern)
                    .unwrap_or_else(|| panic!("Invalid detector regex {}", spec.identifier));

                Some((spec, regex, refs))
            })
            .collect()
    });

    for (spec, regex, refs) in compiled {
        for captures in regex.captures_iter(&compact) {
            if matches!(spec.pattern, Detector::DifferentCollections(_))
                && captures
                    .name("group2")
                    .zip(captures.name("group3"))
                    .is_some_and(|(left, right)| left.as_str() == right.as_str())
            {
                continue;
            }

            if !refs.iter().all(|(first_value, second_value)| {
                captures.name(second_value).is_none_or(|value| {
                    captures
                        .name(first_value)
                        .is_some_and(|original| original.as_str() == value.as_str())
                })
            }) {
                let mut exact = spec.pattern.regex().expect("token detector").to_owned();

                for (name, _) in refs {
                    if let Some(value) = captures.name(name) {
                        exact = exact.replace(
                            &format!("\\{}", name.trim_start_matches("group")),
                            &regex::escape(value.as_str()),
                        );
                    }
                }

                if let Ok(exact) = Regex::new(&exact)
                    && let Some(found) = exact.find(&compact)
                {
                    source.emit(
                        findings,
                        spec.identifier,
                        offsets[found.start()],
                        spec.description,
                    );
                }

                continue;
            }

            let offset = offsets[captures.get(0).unwrap().start()];
            source.emit(findings, spec.identifier, offset, spec.description);
        }
    }

    ast_checks::analyze(source, findings);
    precision::analyze(source, findings);
}

pub fn analyze_project(sources: &[crate::Source], findings: &mut Vec<crate::Finding>) {
    locks::lock_order(sources, findings);
}
