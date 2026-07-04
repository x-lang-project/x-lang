use pest::Parser;

use crate::ParseOptions;
use x_diagnostics::{CompilationFailure, parser_messages};

#[derive(Clone, Copy)]
pub(crate) enum EntryPoint {
    Script,
    Module,
}
