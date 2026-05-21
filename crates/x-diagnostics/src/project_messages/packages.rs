use super::*;

impl ProjectError {
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
}
