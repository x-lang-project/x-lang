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

fn compile_os_native_source(
    config: &OsBuildConfig,
    source: &Path,
    output: &Path,
    optimisation_level: u8,
) -> Result<(), CompilationFailure> {
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let is_cpp = matches!(extension.as_str(), "cc" | "cpp" | "cxx");
    let compiler = if is_cpp { &config.cxx } else { &config.cc };

    let mut arguments = vec![
        format!(
            "--target={}",
            x_utils::target::ProductTarget::FreestandingX86.triple()
        ),
        "-m32".to_string(),
        "-ffreestanding".to_string(),
        "-fno-stack-protector".to_string(),
        "-fno-pic".to_string(),
        "-fno-pie".to_string(),
        "-nostdlib".to_string(),
        // X-OS leaves x87/SSE uninitialised; use the integer ABI.
        "-mno-sse".to_string(),
        "-mno-mmx".to_string(),
        "-msoft-float".to_string(),
        x_utils::optimization::NativeOptimization::new(optimisation_level).argument(),
    ];

    if is_cpp {
        arguments.extend(["-fno-exceptions".to_string(), "-fno-rtti".to_string()]);
    }

    arguments.extend([
        "-c".to_string(),
        source.to_string_lossy().into_owned(),
        "-o".to_string(),
        output.to_string_lossy().into_owned(),
    ]);

    run_checked_tool(compiler, &arguments, OsBuildStage::Compile(source))
}

fn link_kernel(objects: &[PathBuf], config: &OsBuildConfig) -> Result<(), CompilationFailure> {
    let mut link_arguments = vec![
        "-m".to_string(),
        "elf_i386".to_string(),
        "-T".to_string(),
        config.linker_script.to_string_lossy().into_owned(),
        "-nostdlib".to_string(),
        "-o".to_string(),
        config.kernel.to_string_lossy().into_owned(),
    ];
    link_arguments.extend(
        objects
            .iter()
            .map(|object| object.to_string_lossy().into_owned()),
    );

    run_checked_tool(&config.ld, &link_arguments, OsBuildStage::LinkKernel)
}

fn package_iso(config: &OsBuildConfig) -> Result<(), CompilationFailure> {
    if let (Some(grub), Some(iso), Some(iso_directory), Some(grub_cfg)) = (
        config.grub_mkrescue.as_deref(),
        config.iso.as_ref(),
        config.iso_directory.as_ref(),
        config.grub_cfg.as_ref(),
    ) {
        let grub_directory = iso_directory.join("boot").join("grub");
        fs::create_dir_all(&grub_directory)
            .map_err(|error| DriverError::os_iso_directory(grub_directory.display(), error))?;
        fs::copy(
            &config.kernel,
            iso_directory.join("boot").join("kernel.elf"),
        )
        .map_err(DriverError::os_kernel_staging)?;
        fs::copy(grub_cfg, grub_directory.join("grub.cfg"))
            .map_err(DriverError::os_grub_config_staging)?;
        let iso_arguments = vec![
            "-o".to_string(),
            iso.to_string_lossy().into_owned(),
            iso_directory.to_string_lossy().into_owned(),
        ];

        match run_checked_tool(grub, &iso_arguments, OsBuildStage::CreateIso) {
            Ok(()) => x_logging::info!("X-OS ISO: {}", iso.display()),
            Err(error) if error.code() == OperationalDiagnostic::OsToolLaunch.code() => {
                eprintln!(
                    "{}",
                    DriverError::iso_tool_unavailable(config.kernel.display())
                )
            }
            Err(error) => return Err(error),
        }
    }

    Ok(())
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
