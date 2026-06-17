use super::*;

pub(super) fn source_module(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let mut parts: Vec<_> = normalized.split('/').collect();
    let filename = parts.pop().unwrap_or("");
    let split = parts.iter().rposition(|part| *part == "src");
    let (root, nested) = split.map_or_else(
        || (format!("{}@{filename}", parts.join("/")), Vec::new()),
        |index| (parts[..=index].join("/"), parts[index + 1..].to_vec()),
    );
    let mut module = root;

    for part in nested {
        module.push_str("::");
        module.push_str(part);
    }

    if split.is_some() && !matches!(filename, "lib.rs" | "main.rs" | "mod.rs") {
        module.push_str("::");
        module.push_str(filename.strip_suffix(".rs").unwrap_or(filename));
    }

    module
}

pub(super) fn root(module: &str) -> &str {
    module.split("::").next().unwrap_or(module)
}

fn paths(tree: &syn::UseTree, prefix: Vec<String>, output: &mut Vec<Vec<String>>) {
    match tree {
        syn::UseTree::Path(path) => {
            let mut prefix = prefix;
            prefix.push(path.ident.to_string());
            paths(&path.tree, prefix, output);
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                paths(item, prefix.clone(), output);
            }
        }
        syn::UseTree::Name(name) => {
            let mut prefix = prefix;
            prefix.push(name.ident.to_string());
            output.push(prefix);
        }
        syn::UseTree::Rename(rename) => {
            let mut prefix = prefix;
            prefix.push(rename.ident.to_string());
            output.push(prefix);
        }
        syn::UseTree::Glob(_) => output.push(prefix),
    }
}

pub(super) fn targets(record: &Record, modules: &HashSet<String>) -> HashSet<String> {
    let Ok(tree) = syn::parse_str::<syn::UseTree>(&record.shape) else {
        return HashSet::new();
    };

    let mut imports = Vec::new();
    paths(&tree, Vec::new(), &mut imports);

    imports
        .into_iter()
        .filter_map(|path| resolve(&record.module, &path, modules))
        .collect()
}

fn resolve(module: &str, path: &[String], modules: &HashSet<String>) -> Option<String> {
    let mut target: Vec<_> = match path.first()?.as_str() {
        "crate" => vec![root(module).to_owned()],
        "super" | "self" => module.split("::").map(str::to_owned).collect(),
        _ => return None, // Unqualified imports may denote external crates or imported names.
    };

    let mut index = 0;

    while let Some(part) = path.get(index) {
        match part.as_str() {
            "crate" | "self" if index == 0 => {}
            "super" if target.len() > 1 => {
                target.pop();
            }
            "super" => return None,
            _ => break,
        }
        index += 1;
    }

    target.extend(
        path[index..]
            .iter()
            .filter(|part| part.as_str() != "self")
            .cloned(),
    );
    // The suffix can denote an item: retain the longest known module prefix.
    while !target.is_empty() {
        let candidate = target.join("::");

        if modules.contains(&candidate) {
            return Some(candidate);
        }

        target.pop();
    }

    None
}
