use pest::Parser;

use crate::ParseOptions;
use crate::ParsedModule;
use crate::scalar;
use crate::script_entry::normalise_script_entrypoint;
use x_diagnostics::{CompilationFailure, parser_messages};

#[derive(Clone, Copy)]
pub(crate) enum EntryPoint {
    Script,
    Module,
}
