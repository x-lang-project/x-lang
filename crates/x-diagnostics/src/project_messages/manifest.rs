use super::*;

impl ProjectError {
    pub fn package_or_workspace_missing(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("manifest {path} requires [package] or [workspace]"))
            .with_code(ProjectDiagnostic::PackageOrWorkspaceMissing.code())
            .into_failure()
    }

    pub fn runtime_abi_type(
        path: impl fmt::Display,
        runtime_abi_version: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "toolchain.runtime-abi in {path} must be integer {runtime_abi_version}"
        ))
        .with_code(ProjectDiagnostic::RuntimeAbiType.code())
        .into_failure()
    }

    pub fn runtime_abi_unsupported(
        path: impl fmt::Display,
        runtime_abi: impl fmt::Display,
        runtime_abi_version: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("runtime ABI {runtime_abi} in {path} is unsupported; compiler supports ABI {runtime_abi_version}")).with_code(ProjectDiagnostic::RuntimeAbiUnsupported.code()).into_failure()
    }

    pub fn workspace_default_member(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "workspace.default_member in {path} must be a string"
        ))
        .with_code(ProjectDiagnostic::WorkspaceDefaultMember.code())
        .into_failure()
    }

    pub fn dependencies_table(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("[dependencies] in {path} must be a table"))
            .with_code(ProjectDiagnostic::DependenciesTable.code())
            .into_failure()
    }

    pub fn dependency_path_table(
        path: impl fmt::Display,
        name: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "dependency `{name}` in {path} must be a path table"
        ))
        .with_code(ProjectDiagnostic::DependencyPathTable.code())
        .into_failure()
    }
}
