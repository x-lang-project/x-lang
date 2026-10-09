use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::os;
use crate::os::{OsBuildConfig, build_os_artifact};
use crate::{BuildArtifact, BuildOutput};
use x_compile::MirObjectOptions;
use x_compile::ModuleCompileOptions;
use x_diagnostics::CompilationFailure;
use x_diagnostics::DriverError;
use x_link::LinkOptions;
use x_mir::MirProgram;
use x_project::ModuleResolver;
use x_project::manifest::lock::ensure_manifest_lock;
use x_project::manifest::read_utf8;
use x_project::manifest::{LinkSettings, Manifest};
use x_utils::artifacts::ArtifactPaths;
use x_utils::target::ProductTarget;

mod fast;
mod modules;
pub use modules::resolve_manifest_modules;

#[derive(Clone, Copy, Debug, Default)]
pub struct ProjectBuildOptions<'a> {
    pub output: Option<&'a Path>,
    pub optimisation_level: Option<u8>,
    pub save_ir: bool,
    pub target: Option<&'a str>,
}

pub fn build_manifest_project(
    manifest_path: &Path,
    options: ProjectBuildOptions<'_>,
) -> Result<BuildArtifact, x_compile::CompilationFailure> {
    let plan = prepare_manifest_project(manifest_path, options)?;

    execute_manifest_project(plan)
}

pub fn build_manifest_output(
    manifest_path: &Path,
    options: ProjectBuildOptions<'_>,
) -> Result<BuildOutput, CompilationFailure> {
    #[cfg(feature = "c-backend")]
    let started = std::time::Instant::now();
    let plan = prepare_manifest_project(manifest_path, options)?;
    #[cfg(feature = "c-backend")]
    if let Some(result) = fast::try_execute_c(&plan) {
        return result.map(|mut output| {
            if let BuildOutput::C(artifact) = &mut output {
                artifact.elapsed_seconds = started.elapsed().as_secs_f64();
            }

            output
        });
    }

    execute_manifest_project(plan).map(BuildOutput::Native)
}

pub fn check_manifest_project(manifest_path: &Path) -> Result<(), x_compile::CompilationFailure> {
    prepare_manifest_project(manifest_path, ProjectBuildOptions::default()).map(|_| ())
}

pub fn prepare_manifest_jit(
    manifest_path: &Path,
    options: ProjectBuildOptions<'_>,
) -> Result<(MirProgram, u8), CompilationFailure> {
    let plan = prepare_manifest_project(manifest_path, options)?;

    if plan.os_config.is_some()
        || plan
            .target
            .as_deref()
            .is_some_and(|target| ProductTarget::parse(target) != Some(ProductTarget::WindowsMsvc))
        || !plan.link_inputs.libs.is_empty()
        || !plan.link_inputs.lib_paths.is_empty()
        || !plan.link_inputs.include_paths.is_empty()
    {
        return Err(DriverError::jit_project_configuration());
    }

    Ok((plan.mir_program, plan.optimisation_level))
}

pub struct ProjectBuildPlan {
    mir_program: MirProgram,
    project_root: PathBuf,
    name: String,
    os_config: Option<OsBuildConfig>,
    final_exe_path: PathBuf,
    target: Option<String>,
    link_inputs: ManifestLinkInputs,
    optimisation_level: u8,
    save_ir: bool,
}

pub fn prepare_manifest_project(
    manifest_path: &Path,
    options: ProjectBuildOptions<'_>,
) -> Result<ProjectBuildPlan, x_compile::CompilationFailure> {
    let ProjectBuildOptions {
        output,
        optimisation_level,
        save_ir,
        target,
    } = options;

    let manifest = Manifest::load(manifest_path)?;
    let raw = manifest.source();
    let value = manifest.value();

    ensure_manifest_lock(manifest_path, raw, value)?;
    let build = manifest.build_options();
    let effective_optimisation = optimisation_level.or(build.optimization);
    let optimisation_level = effective_optimisation.unwrap_or(0);

    if manifest.package().is_none() {
        let member_manifest = manifest.default_member_manifest()?;

        return prepare_manifest_project(
            &member_manifest,
            ProjectBuildOptions {
                optimisation_level: effective_optimisation,
                ..options
            },
        );
    }

    let package = manifest
        .package()
        .ok_or_else(|| DriverError::manifest_package_missing(manifest_path.display()))?;
    let name = package.name.clone();
    let project_root = manifest_path.parent().unwrap_or(Path::new("."));
    let entry_path = package.entry.clone();
    let mut os_config = manifest.os_options().cloned();

    if let (Some(config), Some(output)) = (os_config.as_mut(), output) {
        config.kernel = output.to_path_buf();
    }

    let no_borrow_check = build.no_borrow_check;

    if !entry_path.exists() {
        return Err(DriverError::manifest_entry_not_found(entry_path.display()));
    }

    let entry_source = read_utf8(&entry_path)?;
    let link_inputs = ManifestLinkInputs::from_settings(manifest.link_options());

    let effective_target = target.or(manifest.target());
    let final_exe_path = project_executable_path(project_root, &name, output);
    let mut resolver = ModuleResolver::new_for_compilation(&entry_path);
    resolve_manifest_modules(&manifest, &mut resolver)?;

    let mir_program = x_compile::compile_owned_module_graph_to_mir(
        resolver,
        &entry_source,
        &entry_path,
        ModuleCompileOptions { no_borrow_check },
    )?;

    Ok(ProjectBuildPlan {
        mir_program,
        project_root: project_root.into(),
        name,
        os_config,
        final_exe_path,
        target: effective_target.map(String::from),
        link_inputs,
        optimisation_level,
        save_ir,
    })
}

#[derive(Default)]
struct ManifestLinkInputs {
    libs: Vec<String>,
    lib_paths: Vec<String>,
    include_paths: Vec<String>,
}

impl ManifestLinkInputs {
    fn from_settings(settings: &LinkSettings) -> Self {
        Self {
            libs: settings.libraries.clone(),
            lib_paths: settings
                .library_paths
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
            include_paths: settings
                .include_paths
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
        }
    }

    fn options<'a>(&'a self, optimisation_level: u8, target_triple: &'a str) -> LinkOptions<'a> {
        LinkOptions {
            libs: &self.libs,
            lib_paths: &self.lib_paths,
            include_paths: &self.include_paths,
            optimisation_level,
            target_triple: Some(target_triple),
            freestanding: false,
            passthrough_arguments: &[],
            reproducible: true,
            gc_sections: false,
        }
    }
}

mod execution;
pub use execution::*;
