use std::fmt;

use x_diagnostics::backend_messages;

#[derive(Debug)]
pub(super) enum EmitError {
    Unsupported(String),
    Internal(String),
}

impl From<&str> for EmitError {
    fn from(value: &str) -> Self {
        Self::Unsupported(value.into())
    }
}

impl From<fmt::Error> for EmitError {
    fn from(_: fmt::Error) -> Self {
        Self::Internal(backend_messages::c_emission_c_source_formatting_failed())
    }
}
