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

use std::{
    fs,
    path::{Path, PathBuf},
};

use x_compile::compile_file_to_object;
use x_link::{LinkOptions, link};

mod build;

mod paths;
mod project;

pub use build::BuildArtifact;
pub use build::BuildOutput;
pub use build::CBuildArtifact;
pub use paths::collect_x_files_recursive;
pub use project::scaffold_new_project;
