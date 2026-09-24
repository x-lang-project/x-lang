use crate::LinkOptions;
use x_diagnostics::CompilationFailure;

#[derive(Debug, PartialEq, Eq)]
pub enum LinkOutcome {
    Linked,
    Unsupported(&'static str),
}

#[cfg(all(windows, target_arch = "x86_64"))]
mod windows;
