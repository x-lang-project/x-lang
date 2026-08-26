use std::path::{Path, PathBuf};

use x_diagnostics::CompilationFailure;
use x_diagnostics::ManifestPosition;
use x_diagnostics::ProjectError;

mod settings;
pub use settings::PackageKind;

pub use x_runtime::RUNTIME_ABI_VERSION;

pub use crate::source::read_utf8;
