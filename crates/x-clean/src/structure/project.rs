mod graph;
mod inventory;
mod module_path;
mod record_kind;
mod type_aliases;
use std::collections::{HashMap, HashSet};

use super::*;
use inventory::Inventory;
use record_kind::{AttributePolicy, RecordKind};

#[derive(Clone)]
struct Record {
    source: usize,
    offset: usize,
    kind: RecordKind,
    name: String,
    shape: String,
    module: String,
}
struct ProjectSink<'a> {
    sources: &'a [Source],
    findings: &'a mut Vec<Finding>,
}
impl ProjectSink<'_> {
    fn hit(self, record: &Record, rule: &str, message: impl Into<String>) {
        self.sources[record.source].emit(self.findings, rule, record.offset, message);
    }
}

pub(super) fn analyze(sources: &[Source], output: &mut Vec<Finding>) {
    let mut rows = Vec::new();

    for (index, text) in sources.iter().enumerate() {
        let module = module_path::source_module(&text.path);
        let mut position = Inventory {
            source: text,
            index,
            module,
            rows: Vec::new(),
        };
        position.visit_file(&text.ast);
        rows.extend(position.rows);
        if text.ast.items.len() >= 40 {
            text.emit(
                output,
                "structure-oversized-module",
                0,
                "File contains at least forty direct Rust items",
            );
        }

        let public = text
            .ast
            .items
            .iter()
            .filter(|position| match position {
                syn::Item::Fn(function) => is_public(&function.vis),
                syn::Item::Struct(text) => is_public(&text.vis),
                syn::Item::Enum(error) => is_public(&error.vis),
                _ => false,
            })
            .count();

        if text.ast.items.len() >= 5 && public * 4 >= text.ast.items.len() * 3 {
            text.emit(
                output,
                "structure-broad-public-surface",
                0,
                "At least 75% of five or more direct items are public",
            );
        }
    }

    for (kind, identifier) in [
        (RecordKind::Record, "structure-same-field-record-shape"),
        (RecordKind::Enum, "structure-same-enum-shape"),
        (RecordKind::Macro, "structure-macro-schema-repetition"),
        (RecordKind::ErrorMap, "structure-error-conversion-schema"),
    ] {
        let mut seen: HashMap<&str, &Record> = HashMap::new();

        for result in rows
            .iter()
            .filter(|result| result.kind == kind && result.shape.len() >= 8)
        {
            if let Some(old) = seen.insert(&result.shape, result) {
                ProjectSink {
                    sources,
                    findings: output,
                }
                .hit(
                    result,
                    identifier,
                    format!(
                        "Same syntactic {kind} schema as {}:{} ({})",
                        sources[old.source].path, old.offset, old.name
                    ),
                );
            }
        }
    }

    for (kind, identifier) in [
        (RecordKind::Cfg, "structure-cfg-policy-drift"),
        (RecordKind::Derive, "structure-derive-policy-drift"),
    ] {
        let mut groups: HashMap<String, Vec<&Record>> = HashMap::new();

        for result in rows.iter().filter(|result| result.kind == kind) {
            let key = if kind == RecordKind::Derive {
                result.name.clone()
            } else {
                format!("{}:{}", result.source, result.module)
            };

            groups.entry(key).or_default().push(result);
        }

        for rs in groups.values() {
            if rs.len() >= 2 && rs.iter().any(|result| result.shape != rs[0].shape) {
                ProjectSink {
                    sources,
                    findings: output,
                }
                .hit(
                    rs[0],
                    identifier,
                    format!(
                        "Sibling/schema policy differs: {:?}",
                        rs.iter().map(|result| &result.shape).collect::<Vec<_>>()
                    ),
                );
            }
        }
    }

    let mut aliases: HashMap<&str, &Record> = HashMap::new();

    for result in rows
        .iter()
        .filter(|result| result.kind == RecordKind::ImportAlias)
    {
        if let Some(pointer) = aliases
            .insert(&result.name, result)
            .filter(|pointer| pointer.module != result.module && pointer.shape != result.shape)
        {
            ProjectSink {
                sources,
                findings: output,
            }
            .hit(
                result,
                "structure-import-alias-collision",
                format!(
                    "Alias {} names {} here and {} in module {}",
                    result.name, result.shape, pointer.shape, pointer.module
                ),
            );
        }
    }

    let mut imports: HashMap<(usize, String, String), Vec<&Record>> = HashMap::new();

    for result in rows.iter().filter(|result| result.kind == RecordKind::Use) {
        let parent = result
            .shape
            .rsplit_once("::")
            .map(|(pointer, _)| pointer.to_owned())
            .unwrap_or_default();

        if !parent.is_empty() {
            imports
                .entry((result.source, result.module.clone(), parent))
                .or_default()
                .push(result);
        }
    }

    for ((_, _, parent), rs) in imports {
        if rs.len() >= 2 {
            ProjectSink {
                sources,
                findings: output,
            }
            .hit(
                rs[1],
                "structure-split-import-source",
                format!("Multiple imports from lexical source {parent}"),
            );
        }
    }

    let mut impls: HashMap<&str, Vec<&Record>> = HashMap::new();

    for result in rows.iter().filter(|result| result.kind == RecordKind::Impl) {
        impls.entry(&result.name).or_default().push(result);
    }

    for (name, rs) in impls {
        let locations: HashSet<_> = rs
            .iter()
            .map(|result| (result.source, &result.module))
            .collect();

        if locations.len() >= 3 {
            ProjectSink {
                sources,
                findings: output,
            }
            .hit(
                rs[0],
                "structure-type-impl-scattering",
                format!(
                    "Inherent target {name} occurs in {} source/module locations; names unresolved",
                    locations.len()
                ),
            );
        }
    }

    type_aliases::analyse(sources, output);
    graph::analyse(sources, output, &rows);
}
