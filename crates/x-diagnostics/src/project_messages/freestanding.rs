use super::*;

impl ProjectError {
    pub fn os_build_section_required(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "an [os] project in {path} requires [build] with freestanding = true"
        ))
        .with_code(ProjectDiagnostic::BuildConfiguration.code())
        .into_failure()
    }

    pub fn build_table_required(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("[build] in {path} must be a table"))
            .with_code(ProjectDiagnostic::BuildConfiguration.code())
            .into_failure()
    }

    pub fn build_boolean_required(
        path: impl fmt::Display,
        key: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("build.{key} in {path} must be a boolean"))
            .with_code(ProjectDiagnostic::BuildConfiguration.code())
            .into_failure()
    }

    pub fn build_optimization_invalid(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "build.optimization in {path} must be an integer from 0 through 3"
        ))
        .with_code(ProjectDiagnostic::BuildConfiguration.code())
        .into_failure()
    }

    pub fn freestanding_required(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "an [os] project in {path} requires build.freestanding = true"
        ))
        .with_code(ProjectDiagnostic::BuildConfiguration.code())
        .into_failure()
    }

    pub fn os_table_required(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("[os] in {path} must be a table"))
            .with_code(ProjectDiagnostic::OsConfiguration.code())
            .into_failure()
    }

    pub fn os_path_escapes_package(
        path: impl fmt::Display,
        key: impl fmt::Display,
        value: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "os.{key} path `{value}` in {path} must stay within the package"
        ))
        .with_code(ProjectDiagnostic::OsPathEscapesPackage.code())
        .into_failure()
    }

    pub fn os_string_required(
        path: impl fmt::Display,
        key: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("os.{key} in {path} must be a string"))
            .with_code(ProjectDiagnostic::OsConfiguration.code())
            .into_failure()
    }

    pub fn os_support_escapes_package(
        path: impl fmt::Display,
        support: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "os.support path `{support}` in {path} must stay within the package"
        ))
        .with_code(ProjectDiagnostic::OsPathEscapesPackage.code())
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

    pub fn target_table(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("[target] in {path} must be a table"))
            .with_code(ProjectDiagnostic::TargetTable.code())
            .into_failure()
    }

    pub fn target_unsupported(
        path: impl fmt::Display,
        triple: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("target triple `{triple}` in {path} is unsupported"))
            .with_code(ProjectDiagnostic::TargetUnsupported.code())
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

    pub fn manifest_array_type(
        path: impl fmt::Display,
        field: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("`{field}` in {path} must be an array of strings"))
            .with_code(ProjectDiagnostic::ManifestStringArray.code())
            .into_failure()
    }

    pub fn manifest_array_item_type(
        path: impl fmt::Display,
        field: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("`{field}` in {path} must contain only strings"))
            .with_code(ProjectDiagnostic::ManifestStringArray.code())
            .into_failure()
    }

    pub fn manifest_array_empty(
        path: impl fmt::Display,
        field: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("`{field}` in {path} must not be empty"))
            .with_code(ProjectDiagnostic::ManifestArrayEmpty.code())
            .into_failure()
    }
}
