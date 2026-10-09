use std::path::{Path, PathBuf};
use std::process::Command;

use crate::artifact::{OutputFormat, RenderOptions, render_build_output};
use x_diagnostics::{CliDiagnostic, CliError, CompilationFailure};
use x_driver::BuildOutput;
use x_driver::manifest_project::{
    ProjectBuildOptions, build_manifest_output, build_manifest_project, check_manifest_project,
    run_os_manifest_artifact,
};
use x_project::source;
use x_utils::project::MANIFEST_FILE_NAME;

#[derive(Clone, Copy)]
pub(super) enum BuildAction {
    Build(OutputFormat),
    Run,
}

impl BuildAction {
    fn render_options(self, emit_llvm: bool) -> RenderOptions {
        RenderOptions {
            format: match self {
                Self::Build(format) => format,
                Self::Run => OutputFormat::Text,
            },
            emit_llvm,
            cached: false,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct BuildOptions<'a> {
    pub output: Option<&'a Path>,
    pub optimisation_level: Option<u8>,
    pub save_ir: bool,
    pub emit_llvm: bool,
    pub target: Option<&'a str>,
    pub no_cache: bool,
}

enum CompilerInput {
    Project(PathBuf),
    Source(PathBuf),
}

impl CompilerInput {
    fn resolve(input: &Path) -> Result<Self, CompilationFailure> {
        if let Some(manifest) = locate_manifest(input) {
            return Ok(Self::Project(manifest));
        }

        if input.is_file() {
            return Ok(Self::Source(input.to_path_buf()));
        }

        Err(CliDiagnostic::source_input_invalid(input.display()))
    }

    fn check(&self) -> Result<(), CompilationFailure> {
        let path = match self {
            Self::Project(manifest) => {
                check_manifest_project(manifest)?;

                manifest
            }
            Self::Source(path) => {
                let source = source::read_utf8(path)?;
                x_compile::check_source(&source, path)?;

                path
            }
        };

        println!("check succeeded: {}", path.display());

        Ok(())
    }

    fn build(&self, options: BuildOptions<'_>) -> Result<BuildOutput, CompilationFailure> {
        match self {
            Self::Project(manifest) => {
                let project_options = ProjectBuildOptions {
                    output: options.output,
                    optimisation_level: options.optimisation_level,
                    save_ir: options.save_ir,
                    target: options.target,
                };

                if options.emit_llvm {
                    build_manifest_project(manifest, project_options).map(BuildOutput::Native)
                } else {
                    build_manifest_output(manifest, project_options)
                }
            }
            Self::Source(path) => build_source(path, options),
        }
    }

    fn run(&self, artifact: &BuildOutput) -> Result<(), CompilationFailure> {
        match self {
            Self::Project(manifest) => {
                if run_os_manifest_artifact(manifest, artifact.path())? {
                    return Ok(());
                }
            }
            Self::Source(_) => {
                if !artifact.executable() {
                    return Err(
                        CliError::NonExecutableArtifact(artifact.path().to_owned()).into_failure()
                    );
                }
            }
        }

        run_executable(artifact.path())
    }
}

pub(super) fn check_path(input: &Path) -> Result<(), CompilationFailure> {
    CompilerInput::resolve(input)?.check()
}

pub(super) fn build_path(
    input: &Path,
    options: BuildOptions<'_>,
    action: BuildAction,
) -> Result<(), CompilationFailure> {
    let input = CompilerInput::resolve(input)?;
    let request = cache::Request::prepare(&input, options);
    let cached = request.as_ref().and_then(cache::Request::load);
    let cache_hit = cached.is_some();
    let artifact = if let Some(artifact) = cached {
        artifact
    } else {
        let _capture = request
            .as_ref()
            .map(|_| x_link::executable_cache::begin_capture());
        let artifact = input.build(options)?;

        if let Some(request) = request.as_ref()
            && let Some(after) = cache::Request::prepare(&input, options)
            && request.same(&after)
        {
            let _ = request.store(&artifact);
        }

        artifact
    };

    let mut render = action.render_options(options.emit_llvm);
    render.cached = cache_hit;
    render_build_output(&artifact, render);

    if matches!(action, BuildAction::Run) {
        input.run(&artifact)?;
    }

    Ok(())
}

fn build_source(
    source_path: &Path,
    options: BuildOptions<'_>,
) -> Result<BuildOutput, CompilationFailure> {
    let source = source::read_utf8(source_path)?;
    let output = options
        .output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| source_path.with_extension(""));

    x_driver::build_output_with_options(
        &source,
        &source_path.to_string_lossy(),
        &output.to_string_lossy(),
        x_driver::BuildOptions {
            emit_llvm: options.emit_llvm,
            optimisation_level: options.optimisation_level.unwrap_or(0),
            save_ir: options.save_ir,
            target_triple: options.target,
            ..Default::default()
        },
    )
}

fn locate_manifest(input: &Path) -> Option<PathBuf> {
    if input.is_file()
        && input
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(MANIFEST_FILE_NAME))
    {
        return Some(input.to_path_buf());
    }

    let candidate = input.join(MANIFEST_FILE_NAME);

    candidate.is_file().then_some(candidate)
}

fn run_executable(path: &Path) -> Result<(), CompilationFailure> {
    let executable = path
        .canonicalize()
        .map_err(|error| CliDiagnostic::executable_locate(path.display(), error))?;
    let status = Command::new(&executable)
        .status()
        .map_err(|error| CliDiagnostic::executable_run(path.display(), error))?;

    if status.success() {
        Ok(())
    } else {
        Err(CliDiagnostic::executable_exit(path.display(), status))
    }
}

mod cache;
pub(crate) mod jit;
