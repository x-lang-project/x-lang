use std::env;
#[cfg(windows)]
use std::iter;
use std::mem;
use std::process;

#[cfg(windows)]
use std::ffi::{CString, c_void};
use std::{path::Path, process::Command, time::Instant};

use super::CompilerInput;
use x_diagnostics::{CliDiagnostic, CliError, CompilationFailure};

fn error(message: impl ToString) -> CompilationFailure {
    CliError::operational(message.to_string()).into_failure()
}

pub(crate) fn run(input: &Path, optimisation_level: Option<u8>) -> Result<(), CompilationFailure> {
    let mut command = Command::new(env::current_exe().map_err(error)?);
    command.arg("jit-worker").arg(input);
    if let Some(level) = optimisation_level {
        command.arg("-O").arg(level.to_string());
    }

    let status = command.status().map_err(error)?;

    if status.success() {
        Ok(())
    } else {
        Err(CliDiagnostic::executable_exit(input.display(), status))
    }
}

pub(crate) fn worker(
    input: &Path,
    optimisation_level: Option<u8>,
) -> Result<(), CompilationFailure> {
    let started = Instant::now();
    let (program, level) = match CompilerInput::resolve(input)? {
        CompilerInput::Source(path) => {
            let source = x_project::source::read_utf8(&path)?;
            let checked = x_compile::compile_source_to_mir(&source, &path, Default::default())?;

            (checked.mir, optimisation_level.unwrap_or(0))
        }
        CompilerInput::Project(path) => x_driver::manifest_project::prepare_manifest_jit(
            &path,
            x_driver::manifest_project::ProjectBuildOptions {
                optimisation_level,
                ..Default::default()
            },
        )?,
    };

    let library = x_link::jit_runtime::library()?;
    let symbols = Symbols::load(&library)?;
    // SAFETY: these C-ABI exports have process-lived runtime and CRT handles.
    let code =
        unsafe { x_compile::jit::execute_mir_jit(&program, level, |name| symbols.address(name)) }?;

    if let Some(flush) = symbols.address("x_jit_flush") {
        // SAFETY: the runtime bridge defines this exact no-argument signature.
        unsafe {
            mem::transmute::<usize, unsafe extern "C" fn()>(flush)();
        }
    }

    eprintln!(
        "JIT execution complete ({:.6} s)",
        started.elapsed().as_secs_f64()
    );
    process::exit(code);
}

struct Symbols {
    #[cfg(windows)]
    libraries: Vec<*mut c_void>,
}
impl Symbols {
    fn load(path: &Path) -> Result<Self, CompilationFailure> {
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;

            let names = [
                path.as_os_str().to_owned(),
                "ucrtbase.dll".into(),
                "msvcrt.dll".into(),
                "vcruntime140.dll".into(),
                "vcruntime140_1.dll".into(),
                "kernel32.dll".into(),
            ];
            let mut libraries = Vec::new();

            for (index, name) in names.into_iter().enumerate() {
                let wide = name.encode_wide().chain(iter::once(0)).collect::<Vec<_>>();
                // SAFETY: the terminated UTF-16 buffer is valid for this call.
                let handle = unsafe { LoadLibraryW(wide.as_ptr()) };

                if handle.is_null() && index == 0 {
                    return Err(error(format!(
                        "could not load JIT runtime: {}",
                        std::io::Error::last_os_error()
                    )));
                }

                if !handle.is_null() {
                    libraries.push(handle);
                }
            }

            Ok(Self { libraries })
        }
        #[cfg(not(windows))]
        {
            let _ = path;

            Err(error("--jit requires Windows x86-64"))
        }
    }

    fn address(&self, name: &str) -> Option<usize> {
        #[cfg(windows)]
        {
            let name = if name == "printf" {
                "x_jit_printf"
            } else {
                name
            };

            let name = CString::new(name).ok()?;

            for handle in &self.libraries {
                // SAFETY: handles were loaded above and remain live.
                let address = unsafe { GetProcAddress(*handle, name.as_ptr().cast()) };

                if !address.is_null() {
                    return Some(address as usize);
                }
            }
        }
        #[cfg(not(windows))]
        let _ = name;

        None
    }
}
#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryW(name: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
}
