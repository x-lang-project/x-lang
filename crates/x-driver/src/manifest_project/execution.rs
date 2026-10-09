use super::*;

pub fn execute_manifest_project(
    plan: ProjectBuildPlan,
) -> Result<BuildArtifact, x_compile::CompilationFailure> {
    if let Some(result) = fast::try_execute(&plan) {
        return result;
    }

    let ProjectBuildPlan {
        mir_program,
        project_root,
        name,
        os_config,
        final_exe_path,
        target,
        link_inputs,
        optimisation_level,
        save_ir,
    } = plan;
    let project_root = project_root.as_path();
    let effective_target = target.as_deref();
    let object_root = os_config
        .as_ref()
        .map(|config| config.kernel.parent().unwrap_or(project_root).to_path_buf())
        .unwrap_or_else(|| project_root.to_path_buf());
    fs::create_dir_all(&object_root)
        .map_err(|error| DriverError::manifest_build_directory(object_root.display(), error))?;
    let compilation = x_compile::compile_mir_program_to_object(
        &mir_program,
        &object_root,
        &name,
        MirObjectOptions {
            optimisation_level,
            target: effective_target,
            save_ir,
        },
    )?;
    let object_path = &compilation.path;

    if let Some(config) = os_config.as_ref() {
        let kernel = build_os_artifact(project_root, object_path, config, optimisation_level)?;

        return Ok(BuildArtifact {
            path: kernel,
            compilation,
            executable: true,
        });
    }

    if let Some(parent) = final_exe_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|error| DriverError::manifest_output_directory(parent.display(), error))?;
    }

    let object_files = [object_path.to_string_lossy().to_string()];
    let exe_path = final_exe_path.to_string_lossy().to_string();

    x_link::link(
        &object_files,
        &exe_path,
        link_inputs.options(optimisation_level, &compilation.target_triple),
    )
    .map_err(|error| error.with_note(DriverError::native_link_note()))?;

    let _ = fs::remove_file(object_path);

    Ok(BuildArtifact {
        path: final_exe_path,
        compilation,
        executable: true,
    })
}

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
