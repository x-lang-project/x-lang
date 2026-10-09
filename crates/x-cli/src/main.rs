#![cfg_attr(
    not(test),
    deny(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unreachable,
        clippy::unwrap_used
    )
)]

use std::any;
use std::panic;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use artifact::OutputFormat;
use compilation::{BuildAction, BuildOptions, build_path, check_path};
use dispatch::*;
use doctor::{doctor, print_version};
use x_diagnostics::CliDiagnostic;
use x_diagnostics::{CompilationFailure, FailureKind};
use x_utils::optimization::NativeOptimization;

mod artifact;
mod cheri;
mod compilation;
mod doctor;

#[derive(Parser)]
#[command(
    name = "x",
    bin_name = "x",
    about = "The X programming language compiler",
    disable_version_flag = true,
    arg_required_else_help = true
)]
struct Cli {
    /// Print compiler, language edition, LLVM, and host target information.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    version: bool,
    #[command(subcommand)]
    command: Option<CommandKind>,
}

#[derive(Subcommand)]
enum CommandKind {
    /// Create a minimal executable X project.
    New { path: PathBuf },
    /// Parse, type-check, borrow-check, and validate executable contracts.
    Check { input: PathBuf },
    /// Build an X source file or x.toml project.
    Build {
        #[arg(default_value = ".")]
        input: PathBuf,
        /// Emit the build artifact and metrics as one JSON object.
        #[arg(long)]
        json: bool,
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
        #[arg(short = 'O', value_parser = clap::value_parser!(u8).range(0..=i64::from(NativeOptimization::MAX)))]
        optimisation_level: Option<u8>,
        #[arg(long)]
        save_ir: bool,
        #[arg(long)]
        emit_llvm: bool,
        #[arg(long)]
        target: Option<String>,
        /// Recompile even if a matching completed executable is cached.
        #[arg(long)]
        no_cache: bool,
    },
    /// Emit a CHERI-RISC-V pure-capability object with a CHERI LLVM SDK.
    CheriBuild {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long)]
        clang: PathBuf,
        #[arg(long)]
        sysroot: PathBuf,
        #[arg(short = 'O', default_value_t = 0, value_parser = clap::value_parser!(u8).range(0..=i64::from(NativeOptimization::MAX)))]
        optimisation_level: u8,
        #[arg(long)]
        save_ir: bool,
    },
    /// Build and run an X source file or x.toml project.
    Run {
        #[arg(default_value = ".")]
        input: PathBuf,
        #[arg(short = 'O', value_parser = clap::value_parser!(u8).range(0..=i64::from(NativeOptimization::MAX)))]
        optimisation_level: Option<u8>,
        #[arg(long)]
        save_ir: bool,
        #[arg(long)]
        emit_llvm: bool,
        /// Recompile even if a matching completed executable is cached.
        #[arg(long)]
        no_cache: bool,
        /// Execute with LLVM JIT instead of linking a native executable.
        #[arg(long, conflicts_with_all = ["save_ir", "emit_llvm"])]
        jit: bool,
    },
    #[command(hide = true)]
    JitWorker {
        input: PathBuf,
        #[arg(short = 'O', value_parser = clap::value_parser!(u8).range(0..=i64::from(NativeOptimization::MAX)))]
        optimisation_level: Option<u8>,
    },
    /// Check the native tools required by this compiler installation.
    Doctor,
}

const EXIT_SUCCESS: u8 = 0;
const EXIT_COMPILE_ERROR: u8 = 1;
const EXIT_INTERNAL_COMPILER_ERROR: u8 = 101;

fn main() -> ExitCode {
    panic::set_hook(Box::new(|_| {}));

    match panic::catch_unwind(main_inner) {
        Ok(code) => code,
        Err(payload) => {
            let detail = panic_payload(&payload);
            let identifier = stable_ice_identifier(detail);
            eprintln!("{}", CliDiagnostic::panic_report(&identifier, detail));

            ExitCode::from(EXIT_INTERNAL_COMPILER_ERROR)
        }
    }
}

fn main_inner() -> ExitCode {
    if let Err(error) = x_logging::initialise() {
        eprintln!("{}", CliDiagnostic::logging_initialization(error));

        return ExitCode::from(EXIT_INTERNAL_COMPILER_ERROR);
    }

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let code = error.exit_code().clamp(0, u8::MAX as i32) as u8;
            let _ = error.print();

            return ExitCode::from(code);
        }
    };

    if let Err(error) = run(cli) {
        eprintln!("{error}");

        return ExitCode::from(exit_code_for_error(&error));
    }

    ExitCode::from(EXIT_SUCCESS)
}

fn panic_payload(payload: &(dyn any::Any + Send)) -> &str {
    payload
        .downcast_ref::<&'static str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or(CliDiagnostic::non_string_panic_payload())
}

fn stable_ice_identifier(detail: &str) -> String {
    let hash = x_utils::fnv1a_64(detail.as_bytes());

    CliDiagnostic::panic_identifier(hash)
}

fn exit_code_for_error(error: &CompilationFailure) -> u8 {
    match error.kind() {
        FailureKind::Internal => EXIT_INTERNAL_COMPILER_ERROR,
        FailureKind::Source => EXIT_COMPILE_ERROR,
    }
}

#[path = "main/dispatch.rs"]
mod dispatch;
