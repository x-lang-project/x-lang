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

#[cfg(feature = "c-backend")]
pub mod c_backend;

mod module_names;

mod sources;

pub use module_names::qualify_module;

pub use sources::SourceCatalog;
pub use x_diagnostics::CompilationFailure;
