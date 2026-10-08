use super::*;

pub(super) fn project_executable_path(
    project_root: &Path,
    name: &str,
    output: Option<&Path>,
) -> PathBuf {
    match output {
        Some(output) => ArtifactPaths::new(output).executable_override(),
        None => ArtifactPaths::new(project_root.join(name)).executable(),
    }
}

pub fn run_os_manifest_artifact(
    manifest_path: &Path,
    kernel: &Path,
) -> Result<bool, CompilationFailure> {
    let manifest = Manifest::load(manifest_path)?;

    if manifest.os_options().is_none() {
        return Ok(false);
    }

    os::run_os_artifact(kernel, manifest.emulator_options())?;

    Ok(true)
}
