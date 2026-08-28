use super::*;

pub(super) fn validate_target(
    path: &Path,
    target: &toml::Value,
    is_os_manifest: bool,
) -> Result<(), CompilationFailure> {
    let target = target
        .as_table()
        .ok_or_else(|| ProjectError::target_table(path.display()))?;
    reject_unknown_keys(path, "target", target, &["triple"])?;
    let triple = required_manifest_string(path, "target", target, "triple")?;
    let spec = x_utils::target::TargetSpec::parse(triple);
    let supported_host = spec.is_some_and(|spec| spec.is_host_product());
    let supported_os = is_os_manifest && spec.is_some_and(|spec| spec.is_os_product());

    if !supported_host && !supported_os {
        return Err(ProjectError::target_unsupported(path.display(), triple));
    }

    Ok(())
}

pub(super) fn validate_toolchain(
    path: &Path,
    root: &toml::value::Table,
) -> Result<(), CompilationFailure> {
    let toolchain = root
        .get("toolchain")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| ProjectError::toolchain_missing(path.display(), RUNTIME_ABI_VERSION))?;
    reject_unknown_keys(path, "toolchain", toolchain, &["runtime-abi"])?;
    let runtime_abi = toolchain
        .get("runtime-abi")
        .and_then(toml::Value::as_integer)
        .ok_or_else(|| ProjectError::runtime_abi_type(path.display(), RUNTIME_ABI_VERSION))?;

    if runtime_abi != i64::from(RUNTIME_ABI_VERSION) {
        return Err(ProjectError::runtime_abi_unsupported(
            path.display(),
            runtime_abi,
            RUNTIME_ABI_VERSION,
        ));
    }

    Ok(())
}
