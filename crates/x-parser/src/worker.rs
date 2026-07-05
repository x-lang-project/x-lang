use std::{cell::RefCell, panic, sync::mpsc, thread};

use crate::ParseOptions;
use crate::pipeline::{self, EntryPoint};
use x_diagnostics::{CompilationFailure, parser_messages};

struct Request {
    input: String,
    options: ParseOptions,
    entrypoint: EntryPoint,
}
