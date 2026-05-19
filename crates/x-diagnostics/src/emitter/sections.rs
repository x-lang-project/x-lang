use std::io::{self, Write};

use super::Emitter;
use super::colours::ColourRole;
use super::frame::{FRAME_GAP, FRAME_INDENT, Glyphs, SectionLine};
use crate::serialise::serializable_diagnostic;
use crate::source_lookup;
use crate::{Diagnostic, DiagnosticFix, FactKind, FileDatabase, FixConfidence};

impl Emitter {
    pub(super) fn emit_compact_tail(
        diagnostic: &Diagnostic,
        mut writer: impl Write,
    ) -> io::Result<()> {
        for count in &diagnostic.notes {
            writeln!(writer, "note: {count}")?;
        }

        if let Some(h) = &diagnostic.help {
            writeln!(writer, "help: {h}")?;
        }

        for fact in &diagnostic.facts {
            writeln!(writer, "reason: {}", fact.message)?;
        }

        for fix in sorted_fixes(diagnostic) {
            writeln!(
                writer,
                "help: {}{}",
                fix_confidence_prefix(fix.confidence),
                fix.message
            )?;
        }

        if let Some(code) = diagnostic.code.as_deref() {
            writeln!(writer, "note: run `x explain {code}` for more information")?;
        }

        Ok(())
    }

    pub(super) fn emit_machine_sections(
        &self,
        database: &FileDatabase,
        diagnostic: &Diagnostic,
        mut writer: impl Write,
    ) -> io::Result<()> {
        let glyphs = Glyphs::for_style(self.style);
        let payload = serializable_diagnostic(database, diagnostic);
        let json = serde_json::to_string_pretty(&payload).map_err(io::Error::other)?;

        self.blank_frame_line(&mut writer, &glyphs)?;
        self.section_line(
            &mut writer,
            &glyphs,
            SectionLine {
                label: "json",
                message: "",
                colour_role: ColourRole::Code,
            },
        )?;
        for line in json.lines() {
            writeln!(
                writer,
                "{FRAME_INDENT}{}{}{}{FRAME_GAP}{}",
                ColourRole::Border.ansi(self.use_colour),
                glyphs.pipe,
                ColourRole::Reset.ansi(self.use_colour),
                line
            )?;
        }

        Ok(())
    }

    pub(super) fn emit_reason_sections(
        &self,
        database: &FileDatabase,
        diagnostic: &Diagnostic,
        mut writer: impl Write,
    ) -> io::Result<()> {
        let glyphs = Glyphs::for_style(self.style);

        for count in &diagnostic.notes {
            self.section_line(
                &mut writer,
                &glyphs,
                SectionLine {
                    label: "note",
                    message: count,
                    colour_role: ColourRole::Note,
                },
            )?;
        }

        for fact in &diagnostic.facts {
            let section = fact_kind_section(fact.kind);
            let mut message = fact.message.clone();

            if let Some(span) = fact.span
                && let Some(file) = database.get(span.source)
            {
                let (line, column) = file.line_column(span.start);
                message.push_str(&format!(" ({}:{line}:{column})", file.path.display()));
            }

            self.section_line(
                &mut writer,
                &glyphs,
                SectionLine {
                    label: section,
                    message: &message,
                    colour_role: ColourRole::Dim,
                },
            )?;
        }

        if let Some(h) = &diagnostic.help {
            self.section_line(
                &mut writer,
                &glyphs,
                SectionLine {
                    label: "reason",
                    message: h,
                    colour_role: ColourRole::Dim,
                },
            )?;
        }

        if !diagnostic.fixes.is_empty() || !diagnostic.suggestions.is_empty() {
            self.blank_frame_line(&mut writer, &glyphs)?;
        }

        Ok(())
    }

    pub(super) fn emit_help_sections(
        &self,
        database: &FileDatabase,
        diagnostic: &Diagnostic,
        mut writer: impl Write,
    ) -> io::Result<()> {
        let glyphs = Glyphs::for_style(self.style);

        for fix in sorted_fixes(diagnostic) {
            let message = format!("{}{}", fix_confidence_prefix(fix.confidence), fix.message);
            self.section_line(
                &mut writer,
                &glyphs,
                SectionLine {
                    label: "help",
                    message: &message,
                    colour_role: ColourRole::Help,
                },
            )?;

            if diagnostic.code.as_deref() == Some("E0006")
                && let Some(name) = source_lookup::unresolved_name_from_diagnostic(diagnostic)
            {
                self.emit_unknown_identifier_patch(&mut writer, &glyphs, &name, &fix.message)?;
            }
        }

        for sug in &diagnostic.suggestions {
            self.emit_suggestion(database, sug, &mut writer)?;
        }

        Ok(())
    }
}

fn fact_kind_section(kind: FactKind) -> &'static str {
    match kind {
        FactKind::Required => "required",
        FactKind::Inferred => "inferred",
        FactKind::Found => "reason",
        FactKind::Note => "context",
    }
}

fn fix_confidence_prefix(confidence: FixConfidence) -> &'static str {
    match confidence {
        FixConfidence::High => "",
        FixConfidence::Medium => "also: ",
        FixConfidence::Low => "if intended: ",
    }
}

fn sorted_fixes(diagnostic: &Diagnostic) -> Vec<&DiagnosticFix> {
    let mut fixes = diagnostic.fixes.iter().collect::<Vec<_>>();
    fixes.sort_by_key(|fix| fix.confidence);

    fixes
}
