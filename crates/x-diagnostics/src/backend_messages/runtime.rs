use std::error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;
use std::path::PathBuf;

use crate::Error;

pub fn failed_to_create_runtime_c_value_error(
    source_path: impl Display,
    error: impl Display,
) -> String {
    Error::new(format!("Failed to create runtime C {source_path}: {error}")).render_detail()
}

pub fn failed_to_write_runtime_c_value_error(
    source_path: impl Display,
    error: impl Display,
) -> String {
    Error::new(format!("Failed to write runtime C {source_path}: {error}")).render_detail()
}

#[derive(Debug)]
pub enum RuntimeSourceError {
    Create { path: PathBuf, cause: io::Error },
    Write { path: PathBuf, cause: io::Error },
}

impl Display for RuntimeSourceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Create { path, cause } => formatter.write_str(
                &failed_to_create_runtime_c_value_error(path.display(), cause),
            ),
            Self::Write { path, cause } => formatter.write_str(
                &failed_to_write_runtime_c_value_error(path.display(), cause),
            ),
        }
    }
}

impl error::Error for RuntimeSourceError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Create { cause, .. } | Self::Write { cause, .. } => Some(cause),
        }
    }
}
