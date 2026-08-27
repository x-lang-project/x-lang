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

pub fn is_canonical_version(version: &str) -> bool {
    let mut parts = version.split('.');

    (0..3).all(|_| {
        parts.next().is_some_and(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part == "0" || !part.starts_with('0'))
        })
    }) && parts.next().is_none()
}
