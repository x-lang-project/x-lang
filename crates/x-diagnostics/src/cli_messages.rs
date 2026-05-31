use std::fmt;

use crate::Error;
use crate::cli;
use crate::{CompilationFailure, OperationalDiagnostic};

pub struct CliDiagnostic;

impl CliDiagnostic {
    pub fn clang_query(path: impl fmt::Display, error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("failed to query {path}: {error}"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn clang_version(
        status: impl fmt::Display,
        stderr: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("clang --version failed with {status}: {stderr}"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn doctor_tools_missing(tools: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "doctor found missing or incompatible required tools: {tools}"
        ))
        .with_code(cli::OPERATIONAL_CODE)
        .into_failure()
    }

    pub fn clock_before_epoch(error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("system clock is before the Unix epoch: {error}"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn probe_directory_create(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("failed to create probe directory {path}: {error}"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn openmp_compiler_start(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("failed to start {path}: {error}"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn openmp_input_missing() -> CompilationFailure {
        Error::new("failed to open the OpenMP compiler probe input")
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn openmp_input_write(error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "failed to write the OpenMP compiler probe: {error}"
        ))
        .with_code(cli::OPERATIONAL_CODE)
        .into_failure()
    }

    pub fn openmp_compiler_wait(error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!(
            "failed to wait for the OpenMP compiler probe: {error}"
        ))
        .with_code(cli::OPERATIONAL_CODE)
        .into_failure()
    }

    pub fn openmp_compile(stderr: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("Clang could not compile and link OpenMP: {stderr}"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn openmp_probe_run(error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("failed to run the linked OpenMP probe: {error}"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn openmp_probe_exit(status: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("linked OpenMP probe exited with {status}"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn probe_directory_remove(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("failed to remove probe directory {path}: {error}"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn command_missing() -> CompilationFailure {
        Error::new("a command is required")
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn source_input_invalid(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("{path} is not an X source or x.toml project"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn package_entry_missing(path: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("{path} has no package.entry"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn executable_locate(
        path: impl fmt::Display,
        error: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("failed to locate executable {path}: {error}"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn executable_run(path: impl fmt::Display, error: impl fmt::Display) -> CompilationFailure {
        Error::new(format!("failed to run {path}: {error}"))
            .with_code(cli::OPERATIONAL_CODE)
            .into_failure()
    }

    pub fn executable_exit(
        path: impl fmt::Display,
        status: impl fmt::Display,
    ) -> CompilationFailure {
        Error::new(format!("{path} exited with {status}"))
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

    pub fn logging_initialization(error: impl fmt::Display) -> String {
        Error::new(format!("compiler logging initialisation failed: {error}"))
            .with_code("ICE-LOGGING-0001")
            .with_note("report the full command, environment, and `x --version`")
            .render_plain()
    }
}

mod cheri;
