use pest::Parser;

use crate::baseline::strip_semantic_features_in_program;

use crate::ParseOptions;
use crate::ParsedModule;
use crate::XParser;
use crate::scalar;
use crate::script_entry::normalise_script_entrypoint;
use x_diagnostics::{CompilationFailure, parser_messages};

#[derive(Clone, Copy)]
pub(crate) enum EntryPoint {
    Script,
    Module,
}
