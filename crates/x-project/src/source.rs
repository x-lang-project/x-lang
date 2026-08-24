use std::{fs::File, io::Read, path::Path};

use x_diagnostics::CompilationFailure;
use x_diagnostics::ProjectError;

pub const MAX_SOURCE_BYTES: usize = x_parser::MAX_SOURCE_BYTES;

pub fn read_utf8(path: &Path) -> Result<String, CompilationFailure> {
    read_with_context(path, &path.display().to_string())
}

pub(crate) fn read_with_context(path: &Path, context: &str) -> Result<String, CompilationFailure> {
    let reader = File::open(path).map_err(|error| ProjectError::source_read(context, error))?;

    read_stream(reader, context)
}

pub(crate) fn read_stream(reader: impl Read, context: &str) -> Result<String, CompilationFailure> {
    // One extra byte distinguishes an exactly full file from a growing stream.
    let mut bytes = Vec::new();

    reader
        .take(MAX_SOURCE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| ProjectError::source_read(context, error))?;
    if bytes.len() > MAX_SOURCE_BYTES {
        return Err(ProjectError::source_size_limit(context, MAX_SOURCE_BYTES));
    }

    String::from_utf8(bytes)
        .map_err(|error| ProjectError::source_encoding(error.utf8_error().valid_up_to(), context))
}
