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

mod cli;
mod failure;
mod internal;
mod link;
mod operational;
mod parser;
mod project;
mod rendering;
mod serialise;
mod settings;
pub use rendering::{
    render_compiler_diagnostic, render_internal_compiler_error, render_stage_error,
    source_span_for_diagnostic, stage_error_diagnostic,
};
pub use settings::{
    diagnostic_colour_choice, diagnostic_style, pretty_diagnostics_enabled,
    set_diagnostic_colour_choice, set_diagnostic_style, set_pretty_diagnostics_enabled,
};
mod source_lookup;
pub use cli::CliError;
pub use failure::{CompilationFailure, FailureKind, StageDiagnostic};
pub use internal::InternalDiagnostic;
pub use link::LinkDiagnostic;
pub use operational::OperationalDiagnostic;
pub use parser::ParserDiagnostic;
pub use project::ProjectDiagnostic;

pub mod alive_messages;
pub mod applicator;
pub mod backend_messages;
pub mod borrow_messages;
pub mod cli_messages;
pub mod contract_messages;
pub mod diagnostic;
pub mod driver_messages;
pub mod emitter;
pub mod explain;
pub mod frontend_messages;
pub mod link_messages;
pub mod macros;
pub mod mir_messages;
pub mod parser_messages;
pub mod project_messages;
pub mod span;
pub mod tooling_messages;

pub use applicator::*;
pub use diagnostic::*;
pub use emitter::*;
pub use explain::*;
pub use span::*;

pub use cli_messages::CliDiagnostic;

pub use driver_messages::{DriverError, OsBuildStage};

pub use project_messages::{ManifestPosition, ProjectError};

pub use tooling_messages::ToolingError;
