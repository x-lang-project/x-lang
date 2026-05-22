use super::*;

impl ProjectError {
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

    pub fn dependencies_table(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("[dependencies] in {path} must be a table"))
            .with_code(ProjectDiagnostic::DependenciesTable.code())
            .into_failure()
    }
}
