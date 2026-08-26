use std::path::Component;
use std::path::Path;

use super::manifest_string_array;

use x_diagnostics::CompilationFailure;
use x_diagnostics::ProjectError;
use x_utils::optimization::NativeOptimization;

mod values;
pub use values::*;
