use std::fmt;

use x_diagnostics::backend_messages;

#[derive(Debug)]
pub(super) enum EmitError {
    Unsupported(String),
    Internal(String),
}
