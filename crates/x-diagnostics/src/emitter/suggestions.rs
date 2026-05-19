use std::cmp::{max, min};
use std::collections::BTreeMap;
use std::io::{self, Write};

use unicode_width::UnicodeWidthStr;

use super::Emitter;
use super::colours::ColourRole;
use super::frame::{Glyphs, PatchLine, PatchMarker, SectionLine};
use super::layout::{byte_to_column, expand_tabs};
use crate::{FileDatabase, FileRecord, SpanEdit, Suggestion, ToolingError};

impl Emitter {
    pub(super) fn emit_unknown_identifier_patch(
        &self,
        mut writer: impl Write,
        glyphs: &Glyphs,
        name: &str,
        fix: &str,
    ) -> io::Result<()> {
        let lower = fix.to_ascii_lowercase();
        let code = if lower.contains("declare") {
            format!("let {name} = <value>")
        } else if lower.contains("import") {
            format!("import some_module.{name}")
        } else {
            return Ok(());
        };

        self.blank_frame_line(&mut writer, glyphs)?;
        self.patch_line(
            &mut writer,
            glyphs,
            PatchLine {
                line_no: 1,
                code: &code,
                marker: '+',
                colour_role: ColourRole::Insert,
            },
        )?;
        self.patch_marker(
            &mut writer,
            glyphs,
            PatchMarker {
                column: 1,
                len: code.len(),
                marker: '+',
                colour_role: ColourRole::Insert,
            },
        )?;
        self.blank_frame_line(&mut writer, glyphs)?;

        Ok(())
    }

    pub(super) fn emit_suggestion(
        &self,
        database: &FileDatabase,
        sug: &Suggestion,
        mut writer: impl Write,
    ) -> io::Result<()> {
        let glyphs = Glyphs::for_style(self.style);
        self.section_line(
            &mut writer,
            &glyphs,
            SectionLine {
                label: "help",
                message: &sug.message,
                colour_role: ColourRole::Help,
            },
        )?;
        self.blank_frame_line(&mut writer, &glyphs)?;

        if sug.edits.is_empty() {
            return Ok(());
        }

        let mut by_file: BTreeMap<_, Vec<&SpanEdit>> = BTreeMap::new();

        for error in &sug.edits {
            by_file.entry(error.span.source).or_default().push(error);
        }

        for (fid, edits) in by_file {
            let file = database.get(fid).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ToolingError::unknown_suggestion_source(fid.0).render_detail(),
                )
            })?;
            self.emit_suggestion_file(file, edits, &mut writer)?;
        }

        Ok(())
    }

    fn emit_suggestion_file(
        &self,
        file: &FileRecord,
        mut edits: Vec<&SpanEdit>,
        mut writer: impl Write,
    ) -> io::Result<()> {
        edits.sort_by_key(|error| error.span.start);

        for edit in &edits {
            if file.source.get(edit.span.start..edit.span.end).is_none() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    ToolingError::invalid_suggestion_span(file.path.display()).render_detail(),
                ));
            }
        }

        let mut minimum_line = usize::MAX;
        let mut maximum_line = 0;

        for error in &edits {
            let (start_line, _) = file.line_column(error.span.start);
            let (end_line, _) =
                file.line_column(error.span.end.saturating_sub(1).max(error.span.start));
            minimum_line = min(minimum_line, start_line);
            maximum_line = max(maximum_line, end_line);
        }

        if minimum_line == usize::MAX {
            return Ok(());
        }

        for line_no in minimum_line..=maximum_line {
            self.emit_suggestion_line(file, &edits, line_no, &mut writer)?;
        }

        Ok(())
    }

    fn emit_suggestion_line(
        &self,
        file: &FileRecord,
        edits: &[&SpanEdit],
        line_no: usize,
        mut writer: impl Write,
    ) -> io::Result<()> {
        let glyphs = Glyphs::for_style(self.style);
        let orig_line = file.line_str(line_no).unwrap_or("");
        let line_clean = orig_line.trim_end_matches(['\r', '\n']);
        let Some(line_index) = line_no.checked_sub(1) else {
            return Ok(());
        };

        let Some(line_start_byte) = file.line_offsets.get(line_index).copied() else {
            return Ok(());
        };

        let line_end_byte = file
            .line_offsets
            .get(line_no)
            .copied()
            .map(|end| end.saturating_sub(1))
            .unwrap_or(file.source.len());

        let line_edits = edits.iter().filter(|error| {
            error.span.start >= line_start_byte && error.span.start <= line_end_byte
        });

        self.patch_line(
            &mut writer,
            &glyphs,
            PatchLine {
                line_no,
                code: &expand_tabs(line_clean),
                marker: ' ',
                colour_role: ColourRole::Source,
            },
        )?;

        for edit in line_edits {
            let column = byte_to_column(file, line_no, edit.span.start);
            let end = edit.span.end.min(line_start_byte + line_clean.len());
            let original_length = byte_to_column(file, line_no, end).saturating_sub(column);
            let replacement_line = edit.replacement.lines().next().unwrap_or("");
            let prefix = line_clean
                .get(..edit.span.start.saturating_sub(line_start_byte))
                .unwrap_or(line_clean);
            let replaced_prefix = expand_tabs(&format!("{prefix}{replacement_line}"));
            let replacement_length = UnicodeWidthStr::width(replaced_prefix.as_str())
                .saturating_sub(column.saturating_sub(1));
            let (marker, colour_role, length) = if edit.span.start == edit.span.end {
                ('+', ColourRole::Insert, replacement_length.max(1))
            } else if edit.replacement.is_empty() {
                ('-', ColourRole::Error, original_length.max(1))
            } else {
                (
                    '~',
                    ColourRole::Replace,
                    max(original_length, replacement_length).max(1),
                )
            };

            self.patch_marker(
                &mut writer,
                &glyphs,
                PatchMarker {
                    column,
                    len: length,
                    marker,
                    colour_role,
                },
            )?;
        }

        self.blank_frame_line(&mut writer, &glyphs)?;

        Ok(())
    }
}
