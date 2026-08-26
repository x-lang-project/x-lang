use super::*;

pub(super) fn resolve_submodule_paths(
    module_name: &str,
    module_path: &Path,
    submodules: &[String],
) -> Result<Vec<(String, PathBuf)>, CompilationFailure> {
    let directory = module_path.parent().unwrap_or(Path::new("."));

    submodules
        .iter()
        .map(|name| {
            let direct = directory.join(format!("{name}.x"));
            let nested = directory.join(name).join("mod.x");
            let path = if direct.is_file() {
                direct
            } else if nested.is_file() {
                nested
            } else {
                return Err(ProjectError::submodule_missing(
                    module_path.display(),
                    direct.display(),
                    nested.display(),
                    name,
                ));
            };

            Ok((format!("{module_name}::{name}"), path))
        })
        .collect()
}

pub(super) fn module_context(module_name: &str, path: &Path) -> String {
    ProjectError::module_context(module_name, path.display())
}

pub(super) fn read_module_source(
    module_name: &str,
    path: &Path,
) -> Result<String, CompilationFailure> {
    source::read_with_context(path, &module_context(module_name, path))
}

pub(super) fn parse_module_declarations(
    source: &str,
    source_path: &Path,
) -> Result<(x_parser::ParsedModule, Vec<Vec<String>>), CompilationFailure> {
    let parsed = x_parser::parse_module_with_spans(source, x_parser::ParseOptions::default())
        .map_err(|error| {
            error
                .with_source(source, source_path)
                .with_note(ProjectError::module_source_note(source_path.display()))
        })?;

    let imports = parsed
        .program
        .statements
        .iter()
        .filter_map(|statement| {
            if let Statement::Import { module, item } = statement {
                Some(
                    module
                        .split("::")
                        .filter(|segment| !segment.is_empty())
                        .chain(iter::once(item.as_str()))
                        .map(str::to_owned)
                        .collect(),
                )
            } else {
                None
            }
        })
        .collect();

    Ok((parsed, imports))
}

mod tree;
