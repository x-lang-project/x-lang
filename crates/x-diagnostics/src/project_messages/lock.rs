use super::*;

impl ProjectError {
    pub fn lock_stale(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("{path} is stale or non-canonical; remove it and rebuild only to accept the manifest or path-dependency changes")).with_code(ProjectDiagnostic::LockStale.code()).into_failure()
    }

    pub fn lock_candidate_create(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "failed to create temporary lockfile under {path}: {error}"
        ))
        .with_code(ProjectDiagnostic::LockCandidateCreate.code())
        .into_failure()
    }

    pub fn lock_candidate_write(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "failed to write and flush temporary lockfile {path}: {error}"
        ))
        .with_code(ProjectDiagnostic::LockCandidateWrite.code())
        .into_failure()
    }

    pub fn lock_install(path: impl fmt::Display, error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("failed to install lockfile {path}: {error}"))
            .with_code(ProjectDiagnostic::LockInstall.code())
            .into_failure()
    }

    pub fn lock_package_missing() -> CompilationFailure {
        Error::new("package manifest disappeared during locking")
            .with_code(ProjectDiagnostic::LockPackageMissing.code())
            .into_failure()
    }

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
