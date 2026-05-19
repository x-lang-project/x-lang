use std::env;
use std::io::{self, Write};

use crate::{Diagnostic, FileDatabase};
use colours::ColourRole;
use frame::Glyphs;
use summary::DiagnosticSummary;

mod colours;
mod frame;
mod layout;
mod sections;
mod source;
mod suggestions;
mod summary;

pub const DEFAULT_DIAGNOSTIC_WIDTH: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticStyle {
    Compact,
    Rich,
    Ascii,
    Unicode,
}

impl DiagnosticStyle {
    fn uses_unicode(self) -> bool {
        matches!(self, Self::Rich | Self::Unicode)
    }

    fn is_compact(self) -> bool {
        matches!(self, Self::Compact)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColourChoice {
    Auto,
    Always,
    Never,
}

pub struct Emitter {
    pub use_colour: bool,
    pub tty_width: usize,
    pub style: DiagnosticStyle,
}

impl Default for Emitter {
    fn default() -> Self {
        let style = crate::diagnostic_style();

        Self {
            use_colour: should_colour(DiagnosticStream::Stderr, crate::diagnostic_colour_choice()),
            tty_width: DEFAULT_DIAGNOSTIC_WIDTH,
            style,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticStream {
    Stdout,
    Stderr,
}

impl Emitter {
    pub fn emit(
        &self,
        database: &FileDatabase,
        diagnostic: &Diagnostic,
        mut writer: impl Write,
    ) -> io::Result<()> {
        let sev_str = diagnostic.severity.name();

        let code_str = diagnostic
            .code
            .as_ref()
            .map(|character| format!("[{character}]"))
            .unwrap_or_default();

        write!(
            writer,
            "{}{}{}",
            ColourRole::for_severity(diagnostic.severity).ansi(self.use_colour),
            sev_str,
            ColourRole::Reset.ansi(self.use_colour)
        )?;
        if !code_str.is_empty() {
            write!(
                writer,
                "{}{}{}",
                ColourRole::Code.ansi(self.use_colour),
                code_str,
                ColourRole::Reset.ansi(self.use_colour)
            )?;
        }

        writeln!(writer, ": {}", diagnostic.message)?;

        if self.style.is_compact() {
            Self::emit_compact_tail(diagnostic, &mut writer)?;
            self.emit_machine_sections(database, diagnostic, &mut writer)?;

            return Ok(());
        }

        self.emit_source_frames(database, diagnostic, &mut writer)?;
        self.emit_reason_sections(database, diagnostic, &mut writer)?;
        self.emit_help_sections(database, diagnostic, &mut writer)?;
        self.emit_machine_sections(database, diagnostic, &mut writer)?;

        if let Some(code) = diagnostic.code.as_deref() {
            let glyphs = Glyphs::for_style(self.style);
            writeln!(
                writer,
                "  {}{}{} more: `x explain {code}`",
                ColourRole::Border.ansi(self.use_colour),
                glyphs.end,
                ColourRole::Reset.ansi(self.use_colour)
            )?;
        }

        Ok(())
    }
}

mod colour;
pub use colour::*;
