use std::fs;

use super::{BuildArtifact, BuildOptions, prepare_object_path};
use x_compile::CompilationFailure;
use x_link::tinycc::{self, LinkOutcome};
use x_project::manifest::Manifest;
use x_utils::artifacts::ArtifactPaths;
use x_utils::project::find_project_manifest;
use x_utils::target::TinyCcWindows;

pub(super) fn try_build(
    source: &str,
    input: &str,
    output: &str,
    options: &BuildOptions<'_>,
) -> Option<Result<BuildArtifact, CompilationFailure>> {
    if options.freestanding || options.emit_object || options.emit_llvm {
        return None;
    }

    let target = options.target_triple.unwrap_or(TinyCcWindows::HOST_TRIPLE);
    let link_options = options.link_options(target);

    match tinycc::configured(&link_options) {
        Ok(false) => None,
        Err(error) => Some(Err(error)),
        Ok(true) => {
            let configured_target = match source_target(input, options) {
                Ok(target) => target,
                Err(error) => return Some(Err(error)),
            };

            if configured_target
                .as_deref()
                .is_some_and(|target| target != TinyCcWindows::HOST_TRIPLE)
            {
                return None;
            }

            match build_fast(source, input, output, options) {
                Ok(artifact) => artifact.map(Ok),
                Err(error) => Some(Err(error)),
            }
        }
    }
}

pub(super) fn source_target(
    input: &str,
    options: &BuildOptions<'_>,
) -> Result<Option<String>, CompilationFailure> {
    if let Some(target) = options.target_triple {
        return Ok(Some(target.to_owned()));
    }

    let Some(path) = find_project_manifest(std::path::Path::new(input)) else {
        return Ok(None);
    };

    Ok(Manifest::load(&path)?.target().map(str::to_owned))
}

fn build_fast(
    source: &str,
    input: &str,
    output: &str,
    options: &BuildOptions<'_>,
) -> Result<Option<BuildArtifact>, CompilationFailure> {
    let output_path = ArtifactPaths::new(output).executable();
    let object = prepare_object_path(output)?;
    let fast_options = BuildOptions {
        target_triple: Some(TinyCcWindows::TRIPLE),
        ..options.clone()
    };
    let mut compilation = x_compile::compile_file_to_object(
        source,
        input,
        &object.to_string_lossy(),
        fast_options.object_options(),
    )?;
    let target = options.target_triple.unwrap_or(TinyCcWindows::HOST_TRIPLE);
    let result = tinycc::link_elf(
        &[object.to_string_lossy().into_owned()],
        &output_path.to_string_lossy(),
        &options.link_options(target),
    );
    let _ = fs::remove_file(&object);

    match result? {
        LinkOutcome::Linked => {
            compilation.target_triple = target.to_owned();

            Ok(Some(BuildArtifact {
                path: output_path,
                compilation,
                executable: true,
            }))
        }
        LinkOutcome::Unsupported(_) => Ok(None),
    }
}
