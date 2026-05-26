use super::*;

impl DriverError {
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
