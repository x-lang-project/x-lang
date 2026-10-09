use std::path::Path;
use std::time::Instant;
use std::{env, fs};

use super::{BuildArtifact, BuildOptions, BuildOutput, CBuildArtifact, fast, prepare_object_path};
use x_compile::c_backend::{CBackendOutput, emit_checked_mir};
use x_compile::{DebugArtifact, MirObjectOptions, SourceMirArtifact, SourceMirOptions};
use x_diagnostics::{CompilationFailure, DriverError};
use x_link::LinkOptions;
use x_link::tinycc::{self, LinkOutcome};
use x_mir::MirProgram;
use x_utils::artifacts::ArtifactPaths;
use x_utils::target::TinyCcWindows;

pub(crate) fn enabled() -> bool {
    env::var_os("X_TCC_BACKEND").is_some_and(|value| value == "c")
}

pub(crate) fn compile_checked(
    mir: &MirProgram,
    output: &Path,
    options: &LinkOptions<'_>,
    debug_artifacts: Vec<DebugArtifact>,
) -> Result<Option<CBuildArtifact>, CompilationFailure> {
    let started = Instant::now();
    let compilation = match emit_checked_mir(mir)? {
        CBackendOutput::Supported(artifact) => artifact,
        CBackendOutput::Unsupported(_) => return Ok(None),
    };

    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|error| DriverError::project_object_directory(parent.display(), error))?;
    }

    match tinycc::compile_c(&compilation.source, &output.to_string_lossy(), options)? {
        LinkOutcome::Unsupported(_) => Ok(None),
        LinkOutcome::Linked => Ok(Some(CBuildArtifact {
            path: output.to_owned(),
            target_triple: TinyCcWindows::HOST_TRIPLE.to_owned(),
            compilation,
            debug_artifacts,
            elapsed_seconds: started.elapsed().as_secs_f64(),
        })),
    }
}

fn native_from_checked(
    checked: SourceMirArtifact,
    output: &str,
    options: &BuildOptions<'_>,
) -> Result<BuildArtifact, CompilationFailure> {
    let object = prepare_object_path(output)?;
    let directory = object
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = object
        .file_stem()
        .and_then(|name| name.to_str())
        .ok_or_else(DriverError::project_object_path_encoding)?;
    let mut compilation = x_compile::compile_mir_program_to_object(
        &checked.mir,
        directory,
        name,
        MirObjectOptions {
            optimisation_level: options.optimisation_level,
            target: Some(TinyCcWindows::HOST_TRIPLE),
            save_ir: false,
        },
    )?;
    compilation.debug_artifacts = checked.debug_artifacts;
    let output_path = ArtifactPaths::new(output).executable();
    let result = x_link::link(
        &[compilation.path.to_string_lossy().into_owned()],
        &output_path.to_string_lossy(),
        options.link_options(&compilation.target_triple),
    );
    let _ = fs::remove_file(&compilation.path);
    result?;

    Ok(BuildArtifact {
        path: output_path,
        compilation,
        executable: true,
    })
}
