use super::*;

impl ProjectError {
    pub fn manifest_tables_required(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("manifest {path} must contain TOML tables"))
            .with_code(ProjectDiagnostic::ManifestStructure.code())
            .into_failure()
    }

    pub fn package_or_workspace_missing(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("manifest {path} requires [package] or [workspace]"))
            .with_code(ProjectDiagnostic::PackageOrWorkspaceMissing.code())
            .into_failure()
    }

    pub fn package_name(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "package.name in {path} must use only ASCII letters, digits, `_`, or `-`"
        ))
        .with_code(ProjectDiagnostic::PackageName.code())
        .into_failure()
    }

    pub fn package_version(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "package.version in {path} must be canonical MAJOR.MINOR.PATCH"
        ))
        .with_code(ProjectDiagnostic::PackageVersion.code())
        .into_failure()
    }

    pub fn language_edition(
        path: impl fmt::Display,
        edition: impl fmt::Display,
        language_edition: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "package.edition `{edition}` in {path} is unsupported; expected `{language_edition}`"
        ))
        .with_code(ProjectDiagnostic::LanguageEdition.code())
        .into_failure()
    }

    pub fn package_entry_path(
        path: impl fmt::Display,
        entry: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "package.entry `{entry}` in {path} must stay within the package"
        ))
        .with_code(ProjectDiagnostic::PackageEntryPath.code())
        .into_failure()
    }

    pub fn package_kind_type(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("package.kind in {path} must be `bin` or `lib`"))
            .with_code(ProjectDiagnostic::PackageKind.code())
            .into_failure()
    }

    pub fn package_kind_unsupported(
        path: impl fmt::Display,
        kind: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "package.kind `{kind}` in {path} must be `bin` or `lib`"
        ))
        .with_code(ProjectDiagnostic::PackageKind.code())
        .into_failure()
    }

    pub fn toolchain_missing(
        path: impl fmt::Display,
        runtime_abi_version: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "package manifest {path} requires [toolchain] with runtime-abi = {runtime_abi_version}"
        ))
        .with_code(ProjectDiagnostic::ToolchainMissing.code())
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

    pub fn link_table(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("[link] in {path} must be a table"))
            .with_code(ProjectDiagnostic::LinkTable.code())
            .into_failure()
    }

    pub fn link_library_name(
        path: impl fmt::Display,
        library: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "link library `{library}` in {path} must be a plain library name"
        ))
        .with_code(ProjectDiagnostic::LinkLibraryName.code())
        .into_failure()
    }

    pub fn link_path_escapes_package(
        path: impl fmt::Display,
        key: impl fmt::Display,
        value: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "link.{key} path `{value}` in {path} must stay within the package"
        ))
        .with_code(ProjectDiagnostic::LinkPathEscapesPackage.code())
        .into_failure()
    }
}
