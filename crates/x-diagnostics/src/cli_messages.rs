use std::fmt;

use crate::Error;
use crate::cli;
use crate::{CompilationFailure, OperationalDiagnostic};

pub struct CliDiagnostic;

impl CliDiagnostic {
    pub fn non_string_panic_payload() -> &'static str {
        "non-string panic payload"
    }
}

mod cheri;
