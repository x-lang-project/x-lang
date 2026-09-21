use x_diagnostics::CompilationFailure;

#[cfg(all(windows, target_arch = "x86_64"))]
mod windows;
