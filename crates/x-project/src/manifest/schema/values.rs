use super::*;

pub(super) fn is_portable_package_relative_path(value: &str) -> bool {
    if value.is_empty() || value.starts_with('/') || value.starts_with('\\') || value.contains(':')
    {
        return false;
    }

    !value
        .split(['/', '\\'])
        .any(|component| component == ".." || component.is_empty())
}

pub(super) fn reject_unknown_keys(
    path: &Path,
    section: &str,
    table: &toml::value::Table,
    allowed: &[&str],
) -> Result<(), CompilationFailure> {
    if let Some(key) = table.keys().find(|key| !allowed.contains(&key.as_str())) {
        Err(ProjectError::unknown_manifest_field(
            path.display(),
            section,
            key,
        ))
    } else {
        Ok(())
    }
}

pub fn required_manifest_string<'a>(
    path: &Path,
    section: &str,
    table: &'a toml::value::Table,
    key: &str,
) -> Result<&'a str, CompilationFailure> {
    table
        .get(key)
        .and_then(toml::Value::as_str)
        .ok_or_else(|| ProjectError::manifest_string_field(path.display(), section, key))
}
