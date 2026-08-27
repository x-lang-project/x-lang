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
