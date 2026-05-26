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

    pub fn native_link_note() -> &'static str {
        "native link failed"
    }

    pub fn iso_tool_unavailable(path: impl fmt::Display) -> String {
        Error::new(format!("warning: grub-mkrescue is unavailable; kernel remains directly bootable by QEMU: {path}")).render_detail()
    }
}

mod os;
