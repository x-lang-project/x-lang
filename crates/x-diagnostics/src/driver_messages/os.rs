use super::*;

impl DriverError {
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
}
