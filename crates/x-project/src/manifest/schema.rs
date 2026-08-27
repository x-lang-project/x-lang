use std::path::Component;
use std::path::Path;

use super::manifest_string_array;

use freestanding::validate_build;
use target::*;
use x_diagnostics::CompilationFailure;
use x_diagnostics::ProjectError;
use x_utils::optimization::NativeOptimization;

mod freestanding;

mod values;
pub use values::*;

mod target;
