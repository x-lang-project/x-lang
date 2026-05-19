use std::path::PathBuf;

use super::SourceIdentifier;

#[derive(Debug)]
pub struct FileRecord {
    pub identifier: SourceIdentifier,
    pub path: PathBuf,
    pub source: String,
    pub line_offsets: Vec<usize>,
}

impl FileRecord {
    fn new(identifier: SourceIdentifier, path: PathBuf, source: String) -> Self {
        let mut line_offsets = vec![0];

        for (index, second_value) in source.bytes().enumerate() {
            if second_value == b'\n' {
                line_offsets.push(index + 1);
            }
        }

        Self {
            identifier,
            path,
            source,
            line_offsets,
        }
    }

    pub fn line_column(&self, byte: usize) -> (usize, usize) {
        let byte = byte.min(self.source.len());
        let position = match self.line_offsets.binary_search(&byte) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };

        let line_start = self.line_offsets.get(position).copied().unwrap_or(0);

        (position + 1, (byte - line_start) + 1)
    }

    pub fn line_str(&self, line: usize) -> Option<&str> {
        let l0 = line.checked_sub(1)?;
        let start = *self.line_offsets.get(l0)?;
        let end = self
            .line_offsets
            .get(l0 + 1)
            .copied()
            .unwrap_or(self.source.len());

        self.source.get(start..end)
    }
}

#[derive(Default)]
pub struct FileDatabase {
    files: Vec<FileRecord>,
}

impl FileDatabase {
    pub fn add_file(&mut self, path: impl Into<PathBuf>, source: String) -> SourceIdentifier {
        let identifier = SourceIdentifier(self.files.len());
        self.files
            .push(FileRecord::new(identifier, path.into(), source));

        identifier
    }

    pub fn get(&self, identifier: SourceIdentifier) -> Option<&FileRecord> {
        self.files.get(identifier.0)
    }
}
