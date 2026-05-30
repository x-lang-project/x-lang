use std::fmt;

use crate::Error;
use crate::cli;
use crate::{CompilationFailure, OperationalDiagnostic};

pub struct CliDiagnostic;

impl CliDiagnostic {
    pub fn package_entry_missing(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("{path} has no package.entry"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn panic_identifier(hash: u64) -> String {
        format!("ICE-{hash:016X}")
    }

    pub fn panic_report(identifier: &str, detail: &str) -> String {
        Error::new("internal compiler error")
            .with_code(identifier)
            .with_note(format!("the compiler panicked: {detail}"))
            .with_note(format!(
                "reduce the input while preserving diagnostic {identifier}"
            ))
            .with_note("report the reduced source, command, `x --version` and diagnostic ID")
            .render_plain()
    }

    pub fn non_string_panic_payload() -> &'static str {
        "non-string panic payload"
    }
}

mod cheri;
