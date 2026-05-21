use super::*;

impl ProjectError {
    pub fn workspace_missing(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("{path} missing [workspace]"))
            .with_code(ProjectDiagnostic::ManifestStructure.code())
            .into_failure()
    }

    pub fn workspace_members_empty(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "{path} [workspace].members must contain at least one package"
        ))
        .with_code(ProjectDiagnostic::ManifestStructure.code())
        .into_failure()
    }

    pub fn workspace_selection_missing(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "{path} [workspace].members has no selectable package"
        ))
        .with_code(ProjectDiagnostic::ManifestStructure.code())
        .into_failure()
    }

    pub fn dependency_manifest_missing(
        path: impl fmt::Display,
        name: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("Dependency `{name}` at {path} is missing x.toml"))
            .with_code(ProjectDiagnostic::ManifestStructure.code())
            .into_failure()
    }

    pub fn package_entry_missing(
        path: impl fmt::Display,
        import_name: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "package `{import_name}` entry is not a source file: {path}"
        ))
        .with_code(ProjectDiagnostic::PackageEntryMissing.code())
        .into_failure()
    }

    pub fn package_entry_resolution(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("cannot resolve package entry {path}: {error}"))
            .with_code(ProjectDiagnostic::PackageEntryResolution.code())
            .into_failure()
    }

    pub fn package_import_ambiguous(
        first: impl fmt::Display,
        second: impl fmt::Display,
        import_name: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "package import `{import_name}` is ambiguous between {first} and {second}"
        ))
        .with_code(ProjectDiagnostic::PackageImportAmbiguous.code())
        .into_failure()
    }

    pub fn workspace_member_manifest_missing(
        path: impl fmt::Display,
        member: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "Workspace member `{member}` is missing x.toml under {path}"
        ))
        .with_code(ProjectDiagnostic::ManifestStructure.code())
        .into_failure()
    }
}
