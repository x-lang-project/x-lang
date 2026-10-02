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

mod artifact;

#[cfg(feature = "c-backend")]
pub mod c_backend;

pub mod jit;

mod module_names;

mod object;
mod options;

mod source_mir;
mod source_object;
mod sources;

pub use artifact::DebugArtifact;

pub use module_names::qualify_module;

pub use options::{IrCompileOptions, ObjectCompileOptions};

pub use source_mir::SourceMirArtifact;
pub use source_mir::SourceMirOptions;

pub use sources::SourceCatalog;
pub use x_diagnostics::CompilationFailure;

use object::object_context;
