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

mod tree;
