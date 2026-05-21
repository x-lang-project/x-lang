use super::*;

impl ProjectError {
    pub fn module_depth_limit(
        module_name: impl fmt::Display,
        maximum_module_depth: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "module `{module_name}` exceeds the maximum project module depth of {maximum_module_depth}"
        ))
        .with_code(ProjectDiagnostic::ModuleDepthLimit.code())
        .into_failure()
    }

    pub fn module_identity_conflict(
        first: impl fmt::Display,
        second: impl fmt::Display,
        module_name: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "module identity `{module_name}` resolves to both {first} and {second}"
        ))
        .with_code(ProjectDiagnostic::ModuleIdentityConflict.code())
        .into_failure()
    }

    pub fn module_count_limit(maximum_project_modules: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "project exceeds the maximum of {maximum_project_modules} modules"
        ))
        .with_code(ProjectDiagnostic::ModuleCountLimit.code())
        .into_failure()
    }

    pub fn source_accounting_overflow() -> CompilationFailure {
        Error::new("project source byte accounting overflowed")
            .with_code(ProjectDiagnostic::ProjectSourceLimit.code())
            .into_failure()
    }

    pub fn project_source_limit(
        maximum_project_source_bytes: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
                        "project module sources exceed the maximum combined size of {maximum_project_source_bytes} bytes"
                    )).with_code(ProjectDiagnostic::ProjectSourceLimit.code()).into_failure()
    }

    pub fn submodule_missing(
        declared_in: impl fmt::Display,
        direct: impl fmt::Display,
        nested: impl fmt::Display,
        submodule: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("submodule `{submodule}` declared in {declared_in} was not found as {direct} or {nested}"))
        .with_code(ProjectDiagnostic::SubmoduleMissing.code())
        .into_failure()
    }

    pub fn module_cycle(node: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("module dependency cycle involving {node}"))
            .with_code(ProjectDiagnostic::ModuleCycle.code())
            .into_failure()
    }

    pub fn source_read(context: impl fmt::Display, error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("failed to read {context}: {error}"))
            .with_code(ProjectDiagnostic::SourceRead.code())
            .into_failure()
    }

    pub fn source_size_limit(
        context: impl fmt::Display,
        maximum_source_bytes: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "{context} exceeds the maximum supported size of {maximum_source_bytes} bytes"
        ))
        .with_code(ProjectDiagnostic::SourceSizeLimit.code())
        .into_failure()
    }

    pub fn source_encoding(
        offset: impl fmt::Display,
        context: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "{context} is not valid UTF-8 (first invalid byte at offset {offset})"
        ))
        .with_code(ProjectDiagnostic::SourceEncoding.code())
        .into_failure()
    }

    pub fn module_context(module_name: &str, path: impl fmt::Display) -> String {
        Error::new(format!("module `{module_name}` at {path}")).render_detail()
    }

    pub fn module_source_note(path: impl fmt::Display) -> String {
        Error::new(format!("module source {path}")).render_detail()
    }
}
