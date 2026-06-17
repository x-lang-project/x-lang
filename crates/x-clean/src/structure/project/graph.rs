use std::collections::{HashMap, HashSet};

use super::{ProjectSink, Record, RecordKind, module_path};
use crate::{Finding, Source};

#[derive(Default)]
struct ModuleGraph {
    edges: HashMap<String, HashSet<String>>,
}

impl ModuleGraph {
    fn connect(&mut self, source: &str, target: String) {
        self.edges
            .entry(source.to_owned())
            .or_default()
            .insert(target);
    }

    fn reaches(&self, at: &str, target: &str, seen: &mut HashSet<String>) -> bool {
        if !seen.insert(at.into()) {
            return false;
        }

        self.edges.get(at).is_some_and(|children| {
            children
                .iter()
                .any(|child| child == target || self.reaches(child, target, seen))
        })
    }

    fn is_cyclic(&self, module: &str) -> bool {
        self.edges.get(module).is_some_and(|children| {
            children
                .iter()
                .any(|child| self.reaches(child, module, &mut HashSet::new()))
        })
    }

    fn incoming(&self, module: &str) -> usize {
        self.edges
            .values()
            .filter(|children| children.contains(module))
            .count()
    }

    fn contains(&self, source: &str, target: &str) -> bool {
        self.edges
            .get(source)
            .is_some_and(|children| children.contains(target))
    }

    fn depth_from(&self, module: &str, seen: &mut HashSet<String>) -> usize {
        if !seen.insert(module.into()) {
            return 0;
        }

        self.edges.get(module).map_or(0, |children| {
            children
                .iter()
                .map(|child| 1 + self.depth_from(child, &mut seen.clone()))
                .max()
                .unwrap_or(0)
        })
    }

    fn depth(&self, module: &str) -> usize {
        self.depth_from(module, &mut HashSet::new())
    }
}

struct ProjectGraphs {
    modules: HashSet<String>,
    imports: ModuleGraph,
    reexports: ModuleGraph,
}

impl ProjectGraphs {
    fn new(rows: &[Record]) -> Self {
        let mut graphs = Self {
            modules: rows.iter().map(|record| record.module.clone()).collect(),
            imports: ModuleGraph::default(),
            reexports: ModuleGraph::default(),
        };

        for record in rows
            .iter()
            .filter(|record| matches!(record.kind, RecordKind::Use | RecordKind::Reexport))
        {
            for target in module_path::targets(record, &graphs.modules) {
                if target != record.module && graphs.modules.contains(&target) {
                    let graph = if record.kind == RecordKind::Use {
                        &mut graphs.imports
                    } else {
                        &mut graphs.reexports
                    };

                    graph.connect(&record.module, target);
                }
            }
        }

        graphs
    }
}

fn report_dependencies(
    sources: &[Source],
    findings: &mut Vec<Finding>,
    rows: &[Record],
    imports: &ModuleGraph,
) {
    for record in rows
        .iter()
        .filter(|record| record.kind == RecordKind::Module)
    {
        if imports.is_cyclic(&record.module) {
            ProjectSink { sources, findings }.hit(
                record,
                "structure-module-dependency-cycle",
                format!(
                    "Lexical import graph cycles through module {}",
                    record.module
                ),
            );
        }

        let incoming = imports.incoming(&record.module);

        if incoming >= 3 {
            ProjectSink { sources, findings }.hit(
                record,
                "structure-module-dependency-hub",
                format!(
                    "Module {} has imports from {incoming} other modules",
                    record.module
                ),
            );
        }
    }
}

fn report_reexports(
    sources: &[Source],
    findings: &mut Vec<Finding>,
    rows: &[Record],
    reexports: &ModuleGraph,
) {
    for record in rows
        .iter()
        .filter(|record| record.kind == RecordKind::Reexport)
    {
        let depth = reexports.depth(&record.module);

        if depth >= 3 {
            ProjectSink { sources, findings }.hit(
                record,
                "structure-public-reexport-chain",
                format!("Lexical public reexport graph has depth {depth}"),
            );
        }
    }
}

fn report_layer_policies(
    sources: &[Source],
    findings: &mut Vec<Finding>,
    rows: &[Record],
    graphs: &ProjectGraphs,
) {
    let policy = regex::Regex::new(r"(?m)^\s*//\s*x-clean-layer:\s*(\w+)\s*<\s*(\w+)\s*$").unwrap();

    for source in sources {
        for capture in policy.captures_iter(&source.text) {
            let root = module_path::root(&module_path::source_module(&source.path)).to_owned();
            let low = format!("{root}::{}", &capture[1]);
            let high = format!("{root}::{}", &capture[2]);

            if let Some(record) = rows.iter().find(|record| {
                graphs.imports.contains(&low, &high)
                    && record.kind == RecordKind::Use
                    && record.module == low
                    && module_path::targets(record, &graphs.modules).contains(&high)
            }) {
                ProjectSink { sources, findings }.hit(record, "structure-module-boundary-crossing",
                    format!("Import {low} -> {high} crosses explicit x-clean-layer policy {low} < {high}"));
            }
        }
    }
}

pub(super) fn analyse(sources: &[Source], findings: &mut Vec<Finding>, rows: &[Record]) {
    let graphs = ProjectGraphs::new(rows);

    report_dependencies(sources, findings, rows, &graphs.imports);
    report_reexports(sources, findings, rows, &graphs.reexports);
    report_layer_policies(sources, findings, rows, &graphs);
}
