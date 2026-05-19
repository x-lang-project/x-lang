use std::io::{self, Write};

use super::colours::ColourRole;
use super::{DiagnosticStyle, Emitter};

pub(super) const FRAME_INDENT: &str = "  ";
pub(super) const FRAME_GAP: &str = "  ";
const PATCH_GUTTER_WIDTH: usize = 3;

pub(super) struct SectionLine<'a> {
    pub(super) label: &'a str,
    pub(super) message: &'a str,
    pub(super) colour_role: ColourRole,
}

pub(super) struct PatchLine<'a> {
    pub(super) line_no: usize,
    pub(super) code: &'a str,
    pub(super) marker: char,
    pub(super) colour_role: ColourRole,
}

pub(super) struct PatchMarker {
    pub(super) column: usize,
    pub(super) len: usize,
    pub(super) marker: char,
    pub(super) colour_role: ColourRole,
}

#[derive(Clone, Copy)]
pub(super) struct Glyphs {
    pub(super) loc: &'static str,
    pub(super) pipe: &'static str,
    pub(super) branch: &'static str,
    pub(super) end: &'static str,
}

impl Glyphs {
    pub(super) fn for_style(style: DiagnosticStyle) -> Self {
        if style.uses_unicode() {
            Self {
                loc: "╭─",
                pipe: "│",
                branch: "├─",
                end: "╰─",
            }
        } else {
            Self {
                loc: "-->",
                pipe: "|",
                branch: "=",
                end: "=",
            }
        }
    }
}

impl Emitter {
    pub(super) fn blank_frame_line(
        &self,
        mut writer: impl Write,
        glyphs: &Glyphs,
    ) -> io::Result<()> {
        writeln!(
            writer,
            "{FRAME_INDENT}{}{}{}",
            ColourRole::Border.ansi(self.use_colour),
            glyphs.pipe,
            ColourRole::Reset.ansi(self.use_colour)
        )
    }

    pub(super) fn section_line(
        &self,
        mut writer: impl Write,
        glyphs: &Glyphs,
        section: SectionLine<'_>,
    ) -> io::Result<()> {
        let SectionLine {
            label,
            message,
            colour_role,
        } = section;
        writeln!(
            writer,
            "{FRAME_INDENT}{}{}{}{FRAME_GAP}{}{}{}: {}",
            ColourRole::Border.ansi(self.use_colour),
            glyphs.branch,
            ColourRole::Reset.ansi(self.use_colour),
            colour_role.ansi(self.use_colour),
            label,
            ColourRole::Reset.ansi(self.use_colour),
            message,
        )
    }

    pub(super) fn patch_line(
        &self,
        mut writer: impl Write,
        glyphs: &Glyphs,
        line: PatchLine<'_>,
    ) -> io::Result<()> {
        let PatchLine {
            line_no,
            code,
            marker,
            colour_role,
        } = line;
        let line_no_text = if line_no == 0 {
            String::new()
        } else {
            line_no.to_string()
        };

        let colour_code = self.use_colour && marker != ' ';
        writeln!(
            writer,
            "{FRAME_INDENT}{}{}{}{FRAME_GAP}{}{:>width$}{} {}{}{}{FRAME_GAP}{}{}{}",
            ColourRole::Border.ansi(self.use_colour),
            glyphs.pipe,
            ColourRole::Reset.ansi(self.use_colour),
            ColourRole::Line.ansi(self.use_colour),
            line_no_text,
            ColourRole::Reset.ansi(self.use_colour),
            ColourRole::Border.ansi(self.use_colour),
            glyphs.pipe,
            ColourRole::Reset.ansi(self.use_colour),
            colour_role.ansi(colour_code),
            code,
            ColourRole::Reset.ansi(colour_code),
            width = PATCH_GUTTER_WIDTH,
        )
    }

    pub(super) fn patch_marker(
        &self,
        mut writer: impl Write,
        glyphs: &Glyphs,
        patch: PatchMarker,
    ) -> io::Result<()> {
        let PatchMarker {
            column,
            len: length,
            marker,
            colour_role,
        } = patch;
        writeln!(
            writer,
            "{FRAME_INDENT}{}{}{}{FRAME_GAP}{:>width$} {}{}{}{FRAME_GAP}{}{}",
            ColourRole::Border.ansi(self.use_colour),
            glyphs.pipe,
            ColourRole::Reset.ansi(self.use_colour),
            "",
            ColourRole::Border.ansi(self.use_colour),
            glyphs.pipe,
            ColourRole::Reset.ansi(self.use_colour),
            " ".repeat(column.saturating_sub(1)),
            colour_role.ansi(self.use_colour).to_string()
                + &marker.to_string().repeat(length)
                + ColourRole::Reset.ansi(self.use_colour),
            width = PATCH_GUTTER_WIDTH,
        )
    }
}
