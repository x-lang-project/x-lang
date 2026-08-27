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

pub(super) fn validate_string_array(
    path: &Path,
    field: &str,
    value: Option<&toml::Value>,
    required: bool,
) -> Result<(), CompilationFailure> {
    let Some(value) = value else {
        return if required {
            Err(ProjectError::required_manifest_field(path.display(), field))
        } else {
            Ok(())
        };
    };

    let array = value
        .as_array()
        .ok_or_else(|| ProjectError::manifest_array_type(path.display(), field))?;

    if array.iter().any(|item| item.as_str().is_none()) {
        return Err(ProjectError::manifest_array_item_type(
            path.display(),
            field,
        ));
    }

    if required && array.is_empty() {
        return Err(ProjectError::manifest_array_empty(path.display(), field));
    }

    Ok(())
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
