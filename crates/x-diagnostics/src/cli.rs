use std::{fmt, path::PathBuf};

use crate::{CompilationFailure, Error};

pub(crate) const OPERATIONAL_CODE: &str = "XCLI0001";

#[derive(Debug, Clone)]
pub enum CliError {
    Operational(String),
    NonExecutableArtifact(PathBuf),
}

impl CliError {
    pub fn operational(detail: impl fmt::Display) -> Self {
        Self::Operational(detail.to_string())
    }

    pub fn into_failure(self) -> CompilationFailure {
        self.into()
    }
}

impl From<CliError> for CompilationFailure {
    fn from(error: CliError) -> Self {
        let detail = match error {
            CliError::Operational(detail) => detail,
            CliError::NonExecutableArtifact(path) => format!(
                "cannot run non-executable artifact {}: --emit-llvm emits an object file; omit --emit-llvm to run the program",
                path.display()
            ),
        };

        Error::new(detail)
            .with_code(OPERATIONAL_CODE)
            .into_failure()
    }
}
