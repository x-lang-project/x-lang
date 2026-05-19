use std::error;
use std::fmt::{self, Display};

use crate::{CompilationFailure, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliveFailureKind {
    PathResolve,
    WslPathLaunch,
    WslPathFailure,
    ToolQuery,
    WslToolQuery,
    VersionQueryFailure,
    VersionUnparseable,
    VersionMismatch,
    ToolLaunch,
    WslToolLaunch,
    DirectoryCreate,
    FileWrite,
    IrNormalizer,
    BorrowNormalizer,
    AttributeNormalizer,
    Logging,
    OptimizerProbe,
    ExampleEnumeration,
    FileRead,
    CompileOriginal,
    ParseComparison,
    CompileComparison,
    Verification,
    ComparisonsFailed,
    UnclosedBlock,
    InvalidTransformedSource,
    InvalidTokenBoundary,
    InvalidAttributeBoundary,
    InvalidAttributeNameBoundary,
    InvalidEditBoundary,
    ExamplesMissing,
    ToolMissing,
    NoComparisons,
    Compilation,
}

pub struct AliveError {
    kind: AliveFailureKind,
    message: Box<Error>,
    cause: Option<Box<dyn error::Error>>,
}
impl AliveError {
    fn new(kind: AliveFailureKind, message: Error) -> Self {
        Self {
            kind,
            message: Box::new(message),
            cause: None,
        }
    }

    fn caused(kind: AliveFailureKind, message: Error, cause: impl error::Error + 'static) -> Self {
        Self {
            kind,
            message: Box::new(message),
            cause: Some(Box::new(cause)),
        }
    }
}

impl Display for AliveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message.render_plain())
    }
}

impl fmt::Debug for AliveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.message.render_plain(), formatter)
    }
}

impl error::Error for AliveError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.cause.as_deref()
    }
}

mod preparation;
pub use preparation::*;
