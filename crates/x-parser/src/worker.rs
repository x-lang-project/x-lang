use std::fmt;

use std::{cell::RefCell, panic, sync::mpsc, thread};

use crate::diagnostics::parser_ice_diagnostic;
use crate::pipeline::{self, EntryPoint};
use crate::{PARSER_STACK_BYTES, ParseOptions, ParsedModule};
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

thread_local! {
    static WORKER: RefCell<Option<Worker>> = const { RefCell::new(None) };
}

pub(super) fn parse(input: &str, options: ParseOptions, entrypoint: EntryPoint) -> ParseResult {
    WORKER
        .try_with(|slot| {
            let mut slot = slot.try_borrow_mut().map_err(worker_failure)?;

            if slot.is_none() {
                *slot = Some(Worker::start()?);
            }

            let Some(worker) = slot.as_ref() else {
                return Err(parser_ice_diagnostic("parser worker was not initialized"));
            };

            let result = worker.request(Request {
                input: input.to_owned(),
                options,
                entrypoint,
            });

            if result
                .as_ref()
                .is_err_and(|error| error.kind() == x_diagnostics::FailureKind::Internal)
            {
                *slot = None;
            }

            result
        })
        .map_err(worker_failure)?
}

impl Worker {
    fn start() -> Result<Self, CompilationFailure> {
        let (requests, incoming) = mpsc::sync_channel::<Request>(0);
        let (outgoing, results) = mpsc::sync_channel(0);

        thread::Builder::new()
            .name("x-parser".to_owned())
            .stack_size(PARSER_STACK_BYTES)
            .spawn(move || {
                while let Ok(request) = incoming.recv() {
                    if outgoing.send(parse_request(request)).is_err() {
                        break;
                    }
                }
            })
            .map_err(worker_failure)?;

        Ok(Self { requests, results })
    }

    fn request(&self, request: Request) -> ParseResult {
        self.requests.send(request).map_err(worker_failure)?;

        self.results.recv().map_err(worker_failure)?
    }
}

fn worker_failure(error: impl fmt::Display) -> CompilationFailure {
    parser_ice_diagnostic(&parser_messages::parser_worker_failure(error))
}

fn parse_request(request: Request) -> ParseResult {
    catch_parser_unwind(|| pipeline::parse(&request.input, request.options, request.entrypoint))
}

fn catch_parser_unwind(parse: impl FnOnce() -> ParseResult + panic::UnwindSafe) -> ParseResult {
    match panic::catch_unwind(parse) {
        Ok(result) => result,
        Err(payload) => {
            let detail = payload
                .downcast_ref::<&'static str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                .unwrap_or(parser_messages::NON_STRING_PARSER_PANIC);

            Err(parser_ice_diagnostic(detail))
        }
    }
}
