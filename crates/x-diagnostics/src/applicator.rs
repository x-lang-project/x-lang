mod plan;

use crate::{Diagnostic, FileDatabase, SourceIdentifier};
use plan::EditPlan;

#[derive(Debug)]
pub struct AppliedFile {
    pub source: SourceIdentifier,
    pub original: String,
    pub modified: String,
    pub edits_applied: usize,
    pub edits_skipped: usize,
}

impl EditPlan<'_> {
    fn apply(self, database: &FileDatabase) -> Vec<AppliedFile> {
        let mut results = Vec::new();

        for (source, edits) in self.per_file {
            let Some(file) = database.get(source) else {
                continue;
            };

            let original = file.source.clone();
            let mut text = String::with_capacity(original.len());
            let mut applied = 0;
            let mut cursor = 0;
            // Coordinates refer to the original source, not the accumulated output.
            for (index, edit) in &edits {
                if self.rejected.contains(index) {
                    continue;
                }

                if let Some(unchanged) = original.get(cursor..edit.span.start) {
                    text.push_str(unchanged);
                    text.push_str(&edit.replacement);
                    cursor = edit.span.end;
                    applied += 1;
                }
            }

            if let Some(tail) = original.get(cursor..) {
                text.push_str(tail);
            }

            results.push(AppliedFile {
                source,
                original,
                modified: text,
                edits_applied: applied,
                edits_skipped: edits.len() - applied,
            });
        }

        results
    }
}
