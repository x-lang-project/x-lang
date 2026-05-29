use super::*;

impl CliDiagnostic {
    pub fn cheri_clang_missing(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("CHERI Clang does not exist: {path}"))
            .with_code(OperationalDiagnostic::CheriClangMissing.code())
            .into_failure()
    }

    pub fn cheri_sysroot_missing(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "CHERI pure-capability sysroot does not exist: {path}"
        ))
        .with_code(OperationalDiagnostic::CheriSysrootMissing.code())
        .into_failure()
    }

    pub fn cheri_input_path_encoding() -> CompilationFailure {
        Error::new("input path is not valid UTF-8")
            .with_code(OperationalDiagnostic::CheriInputPathEncoding.code())
            .into_failure()
    }

    pub fn cheri_output_directory(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("failed to create output directory {path}: {error}"))
            .with_code(OperationalDiagnostic::CheriOutputDirectory.code())
            .into_failure()
    }

    pub fn cheri_ir_write(path: impl fmt::Display, error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("failed to write capability IR {path}: {error}"))
            .with_code(OperationalDiagnostic::CheriIrWrite.code())
            .into_failure()
    }

    pub fn cheri_object_install(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "failed to install certified object {path}: {error}"
        ))
        .with_code(OperationalDiagnostic::CheriObjectInstall.code())
        .into_failure()
    }

    pub fn cheri_ir_marker_missing(required: &str) -> CompilationFailure {
        Error::new(format!(
            "generated IR lacks required pure-capability marker `{required}`"
        ))
        .with_code(OperationalDiagnostic::CheriIrMarkerMissing.code())
        .into_failure()
    }

    pub fn cheri_clang_launch(error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("failed to execute CHERI Clang: {error}"))
            .with_code(OperationalDiagnostic::CheriClangLaunch.code())
            .into_failure()
    }

    pub fn cheri_clang_rejected_ir(
        status: impl fmt::Display,
        stderr: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "CHERI LLVM rejected X capability IR (status {status}):\n{stderr}"
        ))
        .with_code(OperationalDiagnostic::CheriClangRejectedIr.code())
        .into_failure()
    }

    pub fn cheri_readelf_missing(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("{path} is required to certify CheriABI ELF flags"))
            .with_code(OperationalDiagnostic::CheriReadelfMissing.code())
            .into_failure()
    }

    pub fn cheri_object_inspection(error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("failed to inspect CHERI object: {error}"))
            .with_code(OperationalDiagnostic::CheriObjectInspection.code())
            .into_failure()
    }

    pub fn cheri_object_certification(report: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "emitted object lacks CheriABI capability-mode ELF flags:\n{report}"
        ))
        .with_code(OperationalDiagnostic::CheriObjectCertification.code())
        .into_failure()
    }
}
