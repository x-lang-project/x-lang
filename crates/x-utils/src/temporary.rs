use std::error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::process;
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use x_diagnostics::ToolingError;

#[derive(Debug, Clone, Copy)]
enum TemporaryFileError {
    NamesExhausted,
    Closed,
}

impl Display for TemporaryFileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let diagnostic = match self {
            Self::NamesExhausted => ToolingError::temporary_names_exhausted(),
            Self::Closed => ToolingError::temporary_file_closed(),
        };

        formatter.write_str(&diagnostic.render_detail())
    }
}

impl error::Error for TemporaryFileError {}

impl TemporaryFileError {
    fn into_io_error(self) -> io::Error {
        let kind = match self {
            Self::NamesExhausted => io::ErrorKind::AlreadyExists,
            Self::Closed => io::ErrorKind::Other,
        };

        io::Error::new(kind, self)
    }
}

static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
const MAX_NAME_ATTEMPTS: usize = 128;

pub struct TemporaryFile {
    path: PathBuf,
    file: Option<File>,
}

impl TemporaryFile {
    pub fn write_and_sync(&mut self, bytes: &[u8]) -> io::Result<()> {
        use io::Write;

        let file = self
            .file
            .as_mut()
            .ok_or_else(|| TemporaryFileError::Closed.into_io_error())?;
        file.write_all(bytes)?;

        file.sync_all()
    }

    pub fn install(mut self, destination: &Path) -> io::Result<()> {
        // Windows requires closing the file before rename or cleanup.
        drop(self.file.take());

        fs::rename(&self.path, destination)
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = fs::remove_file(&self.path);
    }
}
