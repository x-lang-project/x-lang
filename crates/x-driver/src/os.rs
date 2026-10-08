use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use x_diagnostics::CompilationFailure;
use x_diagnostics::OperationalDiagnostic;
use x_diagnostics::{DriverError, OsBuildStage};

pub use x_project::manifest::{EmulatorSettings as OsRunConfig, OsBuildSettings as OsBuildConfig};

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
