use std::collections::BTreeMap;
use std::io::{self, Write};

use unicode_width::UnicodeWidthStr;

use super::Emitter;
use super::colours::ColourRole;
use super::frame::Glyphs;
use super::layout::expand_tabs;
use crate::{Diagnostic, FileDatabase, FileRecord, Label, LabelStyle, ToolingError};

impl Emitter {
    pub(super) fn emit_source_frames(
        &self,
        database: &FileDatabase,
        diagnostic: &Diagnostic,
        mut writer: impl Write,
    ) -> io::Result<()> {
        let glyphs = Glyphs::for_style(self.style);
        let mut by_file = BTreeMap::new();

        for label in &diagnostic.labels {
            by_file
                .entry(label.span.source)
                .or_insert_with(Vec::new)
                .push(label);
        }

        for (fid, mut labels) in by_file {
            let file = database.get(fid).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ToolingError::unknown_diagnostic_source(fid.0).render_detail(),
                )
            })?;

            labels.sort_by_key(|left_value| left_value.span.start);

            if let Some(first) = labels.first() {
                let (line_no, column_start) = file.line_column(first.span.start);
                writeln!(
                    writer,
                    "  {}{}{} {}{}:{}:{}{}",
                    ColourRole::Border.ansi(self.use_colour),
                    glyphs.loc,
                    ColourRole::Reset.ansi(self.use_colour),
                    ColourRole::Path.ansi(self.use_colour),
                    file.path.display(),
                    line_no,
                    column_start,
                    ColourRole::Reset.ansi(self.use_colour)
                )?;
            }

            self.blank_frame_line(&mut writer, &glyphs)?;

            for label in labels {
                self.emit_source_label(file, label, &glyphs, &mut writer)?;
            }
        }

        Ok(())
    }

    fn emit_source_label(
        &self,
        file: &FileRecord,
        label: &Label,
        glyphs: &Glyphs,
        mut writer: impl Write,
    ) -> io::Result<()> {
        let (line_no, _) = file.line_column(label.span.start);
        let line_source = file.line_str(line_no).unwrap_or("");
        let line_clean = line_source.trim_end_matches(['\r', '\n']);
        let line_width = line_no.to_string().len().max(1);
        let line_start = file.line_offsets.get(line_no - 1).copied().unwrap_or(0);
        let start = line_clean.floor_char_boundary(
            label
                .span
                .start
                .saturating_sub(line_start)
                .min(line_clean.len()),
        );
        let end = line_clean.ceil_char_boundary(
            label
                .span
                .end
                .saturating_sub(line_start)
                .min(line_clean.len())
                .max(start),
        );
        let prefix = expand_tabs(line_clean.get(..start).unwrap_or(""));
        let through_highlight = expand_tabs(line_clean.get(..end).unwrap_or(""));
        let width = UnicodeWidthStr::width(prefix.as_str());
        let span_width = UnicodeWidthStr::width(through_highlight.as_str())
            .saturating_sub(width)
            .max(1);

        writeln!(
            writer,
            "{}{:>width$} {}{}{} {}{}{}",
            ColourRole::Line.ansi(self.use_colour),
            line_no,
            ColourRole::Border.ansi(self.use_colour),
            glyphs.pipe,
            ColourRole::Reset.ansi(self.use_colour),
            ColourRole::Source.ansi(self.use_colour),
            expand_tabs(line_clean),
            ColourRole::Reset.ansi(self.use_colour),
            width = line_width,
        )?;

        let caret_colour = match label.style {
            LabelStyle::Primary => ColourRole::Error,
            LabelStyle::Secondary => ColourRole::Note,
        };
        let marker = match label.style {
            LabelStyle::Primary => "^",
            LabelStyle::Secondary => "-",
        };

        write!(
            writer,
            "{:>line_width$} {}{}{} {}{}{}",
            "",
            ColourRole::Border.ansi(self.use_colour),
            glyphs.pipe,
            ColourRole::Reset.ansi(self.use_colour),
            " ".repeat(width),
            caret_colour.ansi(self.use_colour),
            marker.repeat(span_width),
            line_width = line_width,
        )?;
        if let Some(member) = &label.message {
            write!(writer, " {member}")?;
        }

        writeln!(writer, "{}", ColourRole::Reset.ansi(self.use_colour))?;
        self.blank_frame_line(&mut writer, glyphs)?;

        Ok(())
    }
}
