use crate::LinkOptions;
use x_diagnostics::CompilationFailure;

#[derive(Debug, PartialEq, Eq)]
pub enum LinkOutcome {
    Linked,
    Unsupported(&'static str),
}

#[cfg(all(windows, target_arch = "x86_64"))]
mod windows;

#[cfg(feature = "tinycc-session")]
pub struct Session {
    #[cfg(all(windows, target_arch = "x86_64"))]
    inner: windows::session::Session,
}

pub fn configured(options: &LinkOptions<'_>) -> Result<bool, CompilationFailure> {
    #[cfg(all(windows, target_arch = "x86_64"))]
    return windows::configured(options);
    #[cfg(not(all(windows, target_arch = "x86_64")))]
    {
        let _ = options;

        Ok(false)
    }
}
