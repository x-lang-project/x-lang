use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use x_diagnostics::CompilationFailure;
use x_diagnostics::OperationalDiagnostic;
use x_diagnostics::{DriverError, OsBuildStage};

pub use x_project::manifest::{EmulatorSettings as OsRunConfig, OsBuildSettings as OsBuildConfig};

fn run_checked_tool(
    program: &str,
    arguments: &[String],
    stage: OsBuildStage<'_>,
) -> Result<(), CompilationFailure> {
    let resolved = x_utils::tools::resolve_program(program);
    let output = Command::new(&resolved)
        .args(arguments)
        .output()
        .map_err(|error| DriverError::os_tool_launch(resolved.display(), stage, error))?;

    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Err(DriverError::os_tool_failure(
        output.status,
        stdout,
        stderr,
        stage,
    ))
}

pub fn run_os_artifact(kernel: &Path, config: &OsRunConfig) -> Result<(), CompilationFailure> {
    println!("Booting X-OS with {} (Ctrl-C to stop)", config.command);
    let qemu = x_utils::tools::resolve_program(&config.command);
    let arguments = vec![
        "-kernel".to_string(),
        kernel.to_string_lossy().into_owned(),
        "-m".to_string(),
        config.memory.clone(),
        "-serial".to_string(),
        "stdio".to_string(),
        "-display".to_string(),
        "none".to_string(),
        "-no-reboot".to_string(),
    ];

    let status = Command::new(&qemu)
        .args(arguments)
        .status()
        .map_err(|error| DriverError::os_emulator_launch(qemu.display(), error))?;

    if status.success() {
        Ok(())
    } else {
        Err(DriverError::os_emulator_failure(status))
    }
}
