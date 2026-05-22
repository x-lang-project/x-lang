use std::fmt;

use crate::{CompilationFailure, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationalDiagnostic {
    CheriClangMissing,
    CheriSysrootMissing,
    CheriInputPathEncoding,
    CheriOutputDirectory,
    CheriIrWrite,
    CheriObjectInstall,
    CheriIrMarkerMissing,
    CheriClangLaunch,
    CheriClangRejectedIr,
    CheriReadelfMissing,
    CheriObjectInspection,
    CheriObjectCertification,
    OsToolLaunch,
    OsToolFailure,
    OsBuildDirectory,
    OsRuntimeWrite,
    OsIsoDirectory,
    OsKernelStaging,
    OsGrubConfigStaging,
    OsEmulatorLaunch,
    OsEmulatorFailure,
    ManifestPackageMissing,
    ManifestPackageNameMissing,
    ManifestPackageEntryMissing,
    ManifestEntryNotFound,
    ManifestBuildDirectory,
    ManifestOutputDirectory,
    ManifestBuildBoolean,
    ManifestOsPath,
    ProjectObjectDirectory,
    ProjectObjectPathEncoding,
    ProjectInputNotFound,
    ProjectDirectoryRead,
    ProjectDirectoryEntryRead,
    ProjectInputInspection,
    ProjectDestinationExists,
    ProjectNameInvalid,
    ProjectParentDirectory,
    ProjectDestinationCreate,
    ProjectRollback,
    ProjectSourceDirectory,
    ProjectFileWrite,
    CheriUnsupportedConstruct,
    CheriCertifiedBuildRequired,
    ObjectOutputDirectory,
    IrSave,
    PureFunctionThrows,
    MemoizedFunctionImpure,
    MemoizedFunctionThrows,
    MemoizedMultiFunction,
    MemoizedFunctionNonScalar,
    MultiFunctionDispatchMissing,
    StructInvariantUnsupported,
    ParallelBodyUnsupported,
    ParallelWorkerInputInvalid,
    ParallelWorkerUnknown,
    ParallelWorkerSignatureInvalid,
    ParallelCaptureUnsupported,
    ParallelForeachUnsupported,
    BecomeNotFinal,
}

impl OperationalDiagnostic {
    pub const fn code(self) -> &'static str {
        match self {
            Self::CheriClangMissing => "CHERI0002",
            Self::CheriSysrootMissing => "CHERI0003",
            Self::CheriInputPathEncoding => "CHERI0004",
            Self::CheriOutputDirectory => "CHERI0005",
            Self::CheriIrWrite => "CHERI0006",
            Self::CheriObjectInstall => "CHERI0014",
            Self::CheriIrMarkerMissing => "CHERI0007",
            Self::CheriClangLaunch => "CHERI0008",
            Self::CheriClangRejectedIr => "CHERI0009",
            Self::CheriReadelfMissing => "CHERI0010",
            Self::CheriObjectInspection => "CHERI0011",
            Self::CheriObjectCertification => "CHERI0012",
            Self::OsToolLaunch => "XOS0001",
            Self::OsToolFailure => "XOS0002",
            Self::OsBuildDirectory | Self::OsRuntimeWrite | Self::OsIsoDirectory => "XOS0003",
            Self::OsKernelStaging | Self::OsGrubConfigStaging => "XOS0004",
            Self::OsEmulatorLaunch => "XOS0005",
            Self::OsEmulatorFailure => "XOS0006",
            Self::ManifestPackageMissing
            | Self::ManifestPackageNameMissing
            | Self::ManifestPackageEntryMissing
            | Self::ManifestEntryNotFound
            | Self::ManifestBuildDirectory
            | Self::ManifestOutputDirectory => "XMF0001",
            Self::ManifestBuildBoolean => "XMF0024",
            Self::ManifestOsPath => "XMF0025",
            Self::ProjectObjectDirectory
            | Self::ProjectObjectPathEncoding
            | Self::ProjectInputNotFound
            | Self::ProjectDirectoryRead
            | Self::ProjectDirectoryEntryRead
            | Self::ProjectInputInspection
            | Self::ProjectDestinationExists
            | Self::ProjectNameInvalid
            | Self::ProjectParentDirectory
            | Self::ProjectDestinationCreate
            | Self::ProjectRollback
            | Self::ProjectSourceDirectory
            | Self::ProjectFileWrite => "XPR0001",
            Self::CheriUnsupportedConstruct => "CHERI0013",
            Self::CheriCertifiedBuildRequired => "CHERI0001",
            Self::ObjectOutputDirectory => "C0018",
            Self::IrSave => "C0019",
            Self::PureFunctionThrows
            | Self::MemoizedFunctionImpure
            | Self::MemoizedFunctionThrows
            | Self::MemoizedMultiFunction
            | Self::MemoizedFunctionNonScalar
            | Self::MultiFunctionDispatchMissing
            | Self::StructInvariantUnsupported
            | Self::ParallelBodyUnsupported
            | Self::ParallelWorkerInputInvalid
            | Self::ParallelWorkerUnknown
            | Self::ParallelWorkerSignatureInvalid
            | Self::ParallelCaptureUnsupported
            | Self::ParallelForeachUnsupported
            | Self::BecomeNotFinal => "K0001",
        }
    }

    pub fn into_failure(self, detail: impl fmt::Display) -> CompilationFailure {
        Error::new(detail.to_string())
            .with_code(self.code())
            .into_failure()
    }
}
