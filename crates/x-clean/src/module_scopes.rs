use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use syn::spanned::Spanned;

use crate::{Source, scope::Scope};

struct Edge {
    parent: usize,
    gated_test: bool,
}

fn key(path: &Path) -> String {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    normalized.to_string_lossy().replace('\\', "/")
}

fn explicit_path(module: &syn::ItemMod) -> Option<String> {
    module.attrs.iter().find_map(|attribute| {
        if !attribute.path().is_ident("path") {
            return None;
        }

        if let syn::Meta::NameValue(value) = &attribute.meta
            && let syn::Expr::Lit(literal) = &value.value
            && let syn::Lit::Str(path) = &literal.lit
        {
            Some(path.value())
        } else {
            None
        }
    })
}

struct ModuleCollector<'a> {
    source: &'a Source,
    parent: usize,
    indices: &'a BTreeMap<String, usize>,
    incoming: &'a mut [Vec<Edge>],
}
impl ModuleCollector<'_> {
    fn collect(&mut self, items: &[syn::Item], normal_base: &Path, attribute_base: &Path) {
        for item in items {
            let syn::Item::Mod(module) = item else {
                continue;
            };

            let name = module.ident.to_string();
            let override_path = explicit_path(module);

            if let Some((_, nested)) = &module.content {
                let nested_base = override_path
                    .as_ref()
                    .map_or_else(|| normal_base.join(&name), |path| attribute_base.join(path));
                self.collect(nested, &nested_base, &nested_base);
                continue;
            }

            let candidates = override_path.map_or_else(
                || {
                    vec![
                        normal_base.join(format!("{name}.rs")),
                        normal_base.join(&name).join("mod.rs"),
                    ]
                },
                |path| vec![attribute_base.join(path)],
            );
            let start = module.span().start();
            let gated_test = self.source.ast_scope_at(start.line, start.column + 1) == Scope::Test;

            for candidate in candidates {
                if let Some(&target) = self.indices.get(&key(&candidate)) {
                    self.incoming[target].push(Edge {
                        parent: self.parent,
                        gated_test,
                    });
                }
            }
        }
    }
}

pub(crate) fn classify(sources: &mut [Source]) {
    let indices: BTreeMap<_, _> = sources
        .iter()
        .enumerate()
        .map(|(index, source)| (key(Path::new(&source.path)), index))
        .collect();
    let mut incoming: Vec<Vec<Edge>> = (0..sources.len()).map(|_| vec![]).collect();
    let mut included = vec![false; sources.len()];

    for (parent_index, source) in sources.iter().enumerate() {
        let parent = Path::new(&source.path).parent().unwrap_or(Path::new(""));

        for (path, gated_test) in super::scope_includes::literal_includes(source) {
            if let Some(&target) = indices.get(&key(&parent.join(path))) {
                incoming[target].push(Edge {
                    parent: parent_index,
                    gated_test,
                });
                included[target] = true;
            }
        }
    }

    for (index, source) in sources.iter().enumerate() {
        let file = Path::new(&source.path);
        let parent = file.parent().unwrap_or(Path::new(""));
        let normal_base = match (
            included[index],
            file.file_name().and_then(|name| name.to_str()),
        ) {
            (true, _) => parent.to_path_buf(),
            (_, Some("lib.rs" | "main.rs" | "mod.rs" | "build.rs")) => parent.to_path_buf(),
            _ => parent.join(file.file_stem().unwrap_or_default()),
        };

        ModuleCollector {
            source,
            parent: index,
            indices: &indices,
            incoming: &mut incoming,
        }
        .collect(&source.ast.items, &normal_base, parent);
    }

    let mut test_files: Vec<_> = sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            source.is_explicit_test_file() || (source.is_test_file() && incoming[index].is_empty())
        })
        .collect();

    loop {
        let newly_test: Vec<_> = incoming
            .iter()
            .enumerate()
            .filter(|(index, edges)| {
                !test_files[*index]
                    && !edges.is_empty()
                    && edges
                        .iter()
                        .all(|edge| edge.gated_test || test_files[edge.parent])
            })
            .map(|(index, _)| index)
            .collect();

        if newly_test.is_empty() {
            break;
        }

        for index in newly_test {
            test_files[index] = true;
        }
    }

    for (source, test_only) in sources.iter_mut().zip(test_files) {
        if test_only {
            source.mark_test_file();
        } else {
            source.clear_inferred_test_file();
        }
    }
}
