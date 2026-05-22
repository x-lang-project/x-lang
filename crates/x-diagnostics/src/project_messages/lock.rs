use super::*;

impl ProjectError {
    pub fn lock_dependency_shape(dependency_name: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "dependency `{dependency_name}` is not a validated path table"
        ))
        .with_code(ProjectDiagnostic::LockDependencyShape.code())
        .into_failure()
    }

    pub fn lock_dependency_manifest_missing(
        path: impl fmt::Display,
        dependency_name: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "dependency `{dependency_name}` is missing x.toml under {path}"
        ))
        .with_code(ProjectDiagnostic::LockDependencyManifestMissing.code())
        .into_failure()
    }

    pub fn lock_dependency_parse(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "failed to parse dependency manifest {path}: {error}"
        ))
        .with_code(ProjectDiagnostic::LockDependencyParse.code())
        .into_failure()
    }
}
