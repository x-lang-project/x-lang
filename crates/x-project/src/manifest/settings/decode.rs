use super::*;

pub(super) fn decode_dependencies(
    root: &Path,
    value: Option<&toml::Value>,
) -> Result<Vec<DependencySettings>, CompilationFailure> {
    let Some(table) = value.and_then(toml::Value::as_table) else {
        return Ok(Vec::new());
    };

    table
        .iter()
        .map(|(name, specification)| {
            let declared_path = specification
                .get("path")
                .and_then(toml::Value::as_str)
                .ok_or_else(|| x_diagnostics::ProjectError::lock_dependency_shape(name))?;

            Ok(DependencySettings {
                name: name.clone(),
                path: root.join(declared_path),
                declared_path: declared_path.into(),
            })
        })
        .collect()
}
