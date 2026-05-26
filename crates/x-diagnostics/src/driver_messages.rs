use std::{fmt, path::Path};

use crate::Error;
use crate::{CompilationFailure, OperationalDiagnostic};

#[derive(Debug, Clone, Copy)]
pub enum OsBuildStage<'a> {
    Compile(&'a Path),
    AssembleMultiboot,
    LinkKernel,
    CreateIso,
}

impl fmt::Display for OsBuildStage<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compile(path) => write!(formatter, "compiling {}", path.display()),
            Self::AssembleMultiboot => formatter.write_str("assembling the multiboot header"),
            Self::LinkKernel => formatter.write_str("linking the freestanding kernel"),
            Self::CreateIso => formatter.write_str("creating the bootable ISO"),
        }
    }
}

pub struct DriverError;

impl DriverError {
    pub fn jit_project_configuration() -> CompilationFailure {
        crate::CliError::operational("--jit requires a hosted project without custom link inputs")
            .into_failure()
    }

    pub fn project_object_directory(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "Failed to create object output directory {path}: {error}"
        ))
        .with_code(OperationalDiagnostic::ProjectObjectDirectory.code())
        .into_failure()
    }

    pub fn project_object_path_encoding() -> CompilationFailure {
        Error::new("Invalid object file path")
            .with_code(OperationalDiagnostic::ProjectObjectPathEncoding.code())
            .into_failure()
    }

    pub fn manifest_package_missing(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "Manifest {path} missing required [package] section"
        ))
        .with_code(OperationalDiagnostic::ManifestPackageMissing.code())
        .into_failure()
    }

    pub fn manifest_package_name_missing(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("Manifest {path} missing required package.name"))
            .with_code(OperationalDiagnostic::ManifestPackageNameMissing.code())
            .into_failure()
    }

    pub fn manifest_package_entry_missing(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("Manifest {path} missing required package.entry"))
            .with_code(OperationalDiagnostic::ManifestPackageEntryMissing.code())
            .into_failure()
    }

    pub fn manifest_entry_not_found(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("Project entry file not found: {path}"))
            .with_code(OperationalDiagnostic::ManifestEntryNotFound.code())
            .into_failure()
    }

    pub fn manifest_build_directory(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("failed to create build directory {path}: {error}"))
            .with_code(OperationalDiagnostic::ManifestBuildDirectory.code())
            .into_failure()
    }

    pub fn manifest_output_directory(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("failed to create output directory {path}: {error}"))
            .with_code(OperationalDiagnostic::ManifestOutputDirectory.code())
            .into_failure()
    }

    pub fn manifest_build_boolean(key: &str) -> CompilationFailure {
        Error::new(format!("build.{key} must be a boolean"))
            .with_code(OperationalDiagnostic::ManifestBuildBoolean.code())
            .into_failure()
    }

    pub fn manifest_os_path(key: &str) -> CompilationFailure {
        Error::new(format!("os.{key} is required and must be a string"))
            .with_code(OperationalDiagnostic::ManifestOsPath.code())
            .into_failure()
    }

    pub fn project_input_not_found(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("Path does not exist: {path}"))
            .with_code(OperationalDiagnostic::ProjectInputNotFound.code())
            .into_failure()
    }

    pub fn project_directory_read(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("Failed to read directory {path}: {error}"))
            .with_code(OperationalDiagnostic::ProjectDirectoryRead.code())
            .into_failure()
    }

    pub fn project_directory_entry_read(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("Failed to read directory entry in {path}: {error}"))
            .with_code(OperationalDiagnostic::ProjectDirectoryEntryRead.code())
            .into_failure()
    }

    pub fn project_input_inspection(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("Failed to inspect {path}: {error}"))
            .with_code(OperationalDiagnostic::ProjectInputInspection.code())
            .into_failure()
    }

    pub fn project_destination_exists(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("Destination already exists: {path}"))
            .with_code(OperationalDiagnostic::ProjectDestinationExists.code())
            .into_failure()
    }

    pub fn project_name_invalid(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "Project directory name must use only ASCII letters, digits, `_`, or `-`: {path}"
        ))
        .with_code(OperationalDiagnostic::ProjectNameInvalid.code())
        .into_failure()
    }

    pub fn project_parent_directory(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("Failed to create {path}: {error}"))
            .with_code(OperationalDiagnostic::ProjectParentDirectory.code())
            .into_failure()
    }

    pub fn project_destination_create(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("Failed to create {path}: {error}"))
            .with_code(OperationalDiagnostic::ProjectDestinationCreate.code())
            .into_failure()
    }

    pub fn project_rollback(
        path: impl fmt::Display,
        error: impl fmt::Display,
        cleanup_error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!(
            "{error}; also failed to remove incomplete project {path}: {cleanup_error}"
        ))
        .with_code(OperationalDiagnostic::ProjectRollback.code())
        .into_failure()
    }

    pub fn project_source_directory(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("Failed to create {path}: {error}"))
            .with_code(OperationalDiagnostic::ProjectSourceDirectory.code())
            .into_failure()
    }

    pub fn project_file_write(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("Failed to write {path}: {error}"))
            .with_code(OperationalDiagnostic::ProjectFileWrite.code())
            .into_failure()
    }

    pub fn native_link_note() -> &'static str {
        "native link failed"
    }

    pub fn iso_tool_unavailable(path: impl fmt::Display) -> String {
        Error::new(format!("warning: grub-mkrescue is unavailable; kernel remains directly bootable by QEMU: {path}")).render_detail()
    }
}

mod os;
