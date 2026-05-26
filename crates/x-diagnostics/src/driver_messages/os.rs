use super::*;

impl DriverError {
    pub fn os_tool_launch(
        program: impl fmt::Display,
        stage: OsBuildStage<'_>,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("failed to start `{program}` for {stage}: {error}"))
            .with_code(OperationalDiagnostic::OsToolLaunch.code())
            .into_failure()
    }

    pub fn os_tool_failure(
        status: impl fmt::Display,
        stdout: impl fmt::Display,
        stderr: impl fmt::Display,
        stage: OsBuildStage<'_>,
    ) -> CompilationFailure {
        Error::new(format!(
            "{stage} failed with {status}\nstdout:\n{stdout}\nstderr:\n{stderr}"
        ))
        .with_code(OperationalDiagnostic::OsToolFailure.code())
        .into_failure()
    }

    pub fn os_build_directory(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("failed to create {path}: {error}"))
            .with_code(OperationalDiagnostic::OsBuildDirectory.code())
            .into_failure()
    }

    pub fn os_runtime_write(error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("failed to write freestanding runtime: {error}"))
            .with_code(OperationalDiagnostic::OsRuntimeWrite.code())
            .into_failure()
    }

    pub fn os_iso_directory(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("failed to create {path}: {error}"))
            .with_code(OperationalDiagnostic::OsIsoDirectory.code())
            .into_failure()
    }

    pub fn os_kernel_staging(error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("failed to stage kernel for ISO: {error}"))
            .with_code(OperationalDiagnostic::OsKernelStaging.code())
            .into_failure()
    }

    pub fn os_grub_config_staging(error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("failed to stage grub.cfg: {error}"))
            .with_code(OperationalDiagnostic::OsGrubConfigStaging.code())
            .into_failure()
    }

    pub fn os_emulator_launch(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("failed to launch `{path}`: {error}"))
            .with_code(OperationalDiagnostic::OsEmulatorLaunch.code())
            .into_failure()
    }

    pub fn os_emulator_failure(status: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("QEMU exited with {status}"))
            .with_code(OperationalDiagnostic::OsEmulatorFailure.code())
            .into_failure()
    }
}
