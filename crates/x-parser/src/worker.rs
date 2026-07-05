use std::{cell::RefCell, panic, sync::mpsc, thread};

use crate::ParseOptions;
use crate::ParsedModule;
use crate::pipeline::{self, EntryPoint};
use x_diagnostics::{CompilationFailure, parser_messages};

type ParseResult = Result<ParsedModule, CompilationFailure>;

struct Request {
    input: String,
    options: ParseOptions,
    entrypoint: EntryPoint,
}

struct Worker {
    requests: mpsc::SyncSender<Request>,
    results: mpsc::Receiver<ParseResult>,
}
