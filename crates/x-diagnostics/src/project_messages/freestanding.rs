use super::*;

impl ProjectError {
    pub fn os_string_required(
        path: impl fmt::Display,
        key: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("os.{key} in {path} must be a string"))
            .with_code(ProjectDiagnostic::OsConfiguration.code())
            .into_failure()
    }

    pub fn emulator_table_required(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("[qemu] in {path} must be a table"))
            .with_code(ProjectDiagnostic::EmulatorConfiguration.code())
            .into_failure()
    }

    pub fn emulator_string_required(
        path: impl fmt::Display,
        key: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("qemu.{key} in {path} must be a string"))
            .with_code(ProjectDiagnostic::EmulatorConfiguration.code())
            .into_failure()
    }

    pub fn unknown_manifest_field(
        path: impl fmt::Display,
        section: impl fmt::Display,
        key: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("unknown `{section}.{key}` field in {path}"))
            .with_code(ProjectDiagnostic::UnknownManifestField.code())
            .into_failure()
    }

    pub fn manifest_string_field(
        path: impl fmt::Display,
        section: impl fmt::Display,
        key: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("`{section}.{key}` in {path} must be a string"))
            .with_code(ProjectDiagnostic::ManifestStringField.code())
            .into_failure()
    }

    pub fn required_manifest_field(
        path: impl fmt::Display,
        field: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("`{field}` is required in {path}"))
            .with_code(ProjectDiagnostic::RequiredManifestField.code())
            .into_failure()
    }
}
