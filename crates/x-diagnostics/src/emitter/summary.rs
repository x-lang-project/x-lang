use std::io::{self, Write};

use super::colours::ColourRole;
use crate::{Diagnostic, Severity};

#[derive(Default)]
pub(super) struct DiagnosticSummary {
    errors: usize,
    warnings: usize,
}

impl DiagnosticSummary {
    pub(super) fn record(&mut self, diagnostic: &Diagnostic) {
        match diagnostic.severity {
            Severity::Error => self.errors += 1,
            Severity::Warning => self.warnings += 1,
            Severity::Note | Severity::Help => {}
        }
    }

    pub(super) fn emit(&self, use_colour: bool, mut writer: impl Write) -> io::Result<()> {
        if self.errors == 0 && self.warnings == 0 {
            return Ok(());
        }

        let role = if self.errors > 0 {
            ColourRole::Error
        } else {
            ColourRole::Warning
        };

        write!(writer, "{}", role.ansi(use_colour))?;
        if self.errors > 0 {
            write!(
                writer,
                "error: {} previous error{}",
                self.errors,
                plural(self.errors)
            )?;

            if self.warnings > 0 {
                write!(
                    writer,
                    "; {} warning{}",
                    self.warnings,
                    plural(self.warnings)
                )?;
            }
        } else {
            write!(
                writer,
                "warning: {} warning{}",
                self.warnings,
                plural(self.warnings)
            )?;
        }
        writeln!(writer, "{}", ColourRole::Reset.ansi(use_colour))
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}
