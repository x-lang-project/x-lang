use std::fs;

use super::{BuildArtifact, BuildOptions, prepare_object_path};
use x_compile::CompilationFailure;
use x_link::tinycc::{self, LinkOutcome};
use x_project::manifest::Manifest;
use x_utils::artifacts::ArtifactPaths;
use x_utils::project::find_project_manifest;
use x_utils::target::TinyCcWindows;

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
