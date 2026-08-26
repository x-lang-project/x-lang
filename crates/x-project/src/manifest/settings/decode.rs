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

pub(super) fn decode_link(root: &Path, table: Option<&toml::value::Table>) -> LinkSettings {
    let Some(table) = table else {
        return LinkSettings::default();
    };

    let unique = |key| {
        let mut values = Vec::new();

        for value in manifest_string_array(table, key) {
            if !value.is_empty() && !values.contains(&value) {
                values.push(value);
            }
        }

        values
    };

    LinkSettings {
        libraries: unique("libs"),
        library_paths: unique("lib_paths")
            .into_iter()
            .map(|path| root.join(path))
            .collect(),
        include_paths: unique("include_paths")
            .into_iter()
            .map(|path| root.join(path))
            .collect(),
    }
}

pub(super) fn decode_os(
    path: &Path,
    root: &Path,
    table: &toml::value::Table,
) -> Result<OsBuildSettings, CompilationFailure> {
    let required =
        |key| required_manifest_string(path, "os", table, key).map(|path| root.join(path));
    let optional = |key| {
        table
            .get(key)
            .and_then(toml::Value::as_str)
            .map(|path| root.join(path))
    };
    let tool = |key, fallback: &str| {
        table
            .get(key)
            .and_then(toml::Value::as_str)
            .unwrap_or(fallback)
            .to_owned()
    };

    Ok(OsBuildSettings {
        boot_asm: required("boot_asm")?,
        crt0: required("crt0")?,
        support: manifest_string_array(table, "support")
            .into_iter()
            .map(|path| root.join(path))
            .collect(),
        linker_script: required("linker_script")?,
        kernel: required("kernel")?,
        iso: optional("iso"),
        iso_directory: optional("iso_dir"),
        grub_cfg: optional("grub_cfg"),
        nasm: tool("nasm", "nasm"),
        cc: tool("cc", "clang"),
        cxx: tool("cxx", "clang++"),
        ld: tool("ld", "ld.lld"),
        grub_mkrescue: table
            .get("grub_mkrescue")
            .and_then(toml::Value::as_str)
            .map(String::from),
    })
}
