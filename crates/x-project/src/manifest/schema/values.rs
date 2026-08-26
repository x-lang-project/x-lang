use super::*;

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
