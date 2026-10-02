use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use inkwell::targets::{TargetMachine, TargetTriple};

use crate::CompilationFailure;
use x_codegen::ObjectEmissionOptions;
use x_diagnostics::{StageDiagnostic, contract_messages};
use x_logging::info;
use x_project::manifest::Manifest;
use x_utils::artifacts::ArtifactPaths;
use x_utils::project::find_project_manifest;

fn find_target_triple_from_manifest(
    input_path: &str,
) -> Result<Option<String>, CompilationFailure> {
    let Some(path) = find_project_manifest(Path::new(input_path)) else {
        return Ok(None);
    };

    let manifest = Manifest::load(&path)?;

    Ok(manifest.target().map(String::from))
}

fn archive_ir_copy(
    input_path: &str,
    optimisation_level: u8,
    ir_content: &str,
) -> Result<(), CompilationFailure> {
    let save_name = ArtifactPaths::new(input_path).archived_ir(optimisation_level);

    fs::write(&save_name, ir_content)
        .map_err(|error| contract_messages::ir_save(save_name.display(), error))?;
    info!("Saved IR to {}", save_name.display());

    Ok(())
}
