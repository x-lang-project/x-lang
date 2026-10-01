use std::fmt;

use std::env;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::LinkOutcome;
use crate::{LinkOptions, NativeToolchain};
use library::Library;
use prepared::PreparedRuntime;
use x_diagnostics::{CompilationFailure, LinkDiagnostic};
use x_utils::target::TinyCcWindows;

mod cache;
#[cfg(feature = "tinycc-session")]
pub(super) mod generated;
mod library;
mod prepared;
#[cfg(feature = "tinycc-session")]
pub(super) mod session;
const UCRT: &str = x_runtime::source::WINDOWS_UCRT;

// TinyCC has process-global state; keep states and callbacks inside this lock.
static TINYCC: Mutex<()> = Mutex::new(());

pub(super) fn failure(detail: impl fmt::Display) -> CompilationFailure {
    LinkDiagnostic::TinyCcFailure.into_failure(detail)
}

pub(super) fn supported_options(options: &LinkOptions<'_>) -> bool {
    options.optimisation_level == 0
        && !options.freestanding
        && !options.gc_sections
        && options.libs.is_empty()
        && options.lib_paths.is_empty()
        && options.include_paths.is_empty()
        && options.passthrough_arguments.is_empty()
        && options
            .target_triple
            .is_none_or(|target| target == TinyCcWindows::HOST_TRIPLE)
        && ["CPATH", "C_INCLUDE_PATH", "CPLUS_INCLUDE_PATH"]
            .iter()
            .all(|name| env::var_os(name).is_none_or(|value| value.is_empty()))
        && env::var_os("X_RUNTIME_CACHE_DIR")
            .unwrap_or_else(|| env::temp_dir().into_os_string())
            .is_ascii()
}

pub(super) struct Configuration {
    directory: PathBuf,
    object_type: OutputKind,
    ucrt: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OutputKind {
    Executable,
    Object,
}

impl OutputKind {
    pub(super) const fn abi(self) -> c_int {
        match self {
            Self::Executable => 2,
            Self::Object => 3,
        }
    }
}

fn header_object_output_type(header: &str) -> Option<c_int> {
    header.lines().find_map(|line| {
        let mut words = line.split_whitespace();

        (words.next() == Some("#define") && words.next() == Some("TCC_OUTPUT_OBJ"))
            .then(|| words.next()?.parse::<c_int>().ok())
            .flatten()
    })
}

fn object_output_type(header: &str) -> Result<OutputKind, CompilationFailure> {
    match header_object_output_type(header) {
        Some(3) => Ok(OutputKind::Object),
        Some(4) => Err(failure(
            "TinyCC 0.9.27 is unsupported by the in-process linker; use a current development package (TCC_OUTPUT_OBJ=3)",
        )),
        _ => Err(failure("unsupported libtcc object-output ABI")),
    }
}

fn validate_options_abi(header: &str) -> Result<(), CompilationFailure> {
    if header
        .split_whitespace()
        .zip(header.split_whitespace().skip(1))
        .any(|(value_type, name)| value_type == "int" && name.starts_with("tcc_set_options("))
    {
        Ok(())
    } else {
        Err(failure("TinyCC requires int-returning tcc_set_options"))
    }
}

impl Configuration {
    fn from_environment() -> Result<Option<Self>, CompilationFailure> {
        let Some(directory) = env::var_os("X_TCC_DIR") else {
            return Ok(None);
        };

        let directory = PathBuf::from(directory).canonicalize().map_err(failure)?;
        let header = fs::read_to_string(directory.join("libtcc/libtcc.h")).map_err(failure)?;
        // TinyCC 0.9.27 overflows its stack on the C11 runtime; reject it before loading.
        let object_type = object_output_type(&header)?;
        validate_options_abi(&header)?;
        if !directory.join("libtcc.dll").is_file()
            || !directory.join("include").is_dir()
            || !directory.join("lib").is_dir()
        {
            return Err(failure(
                "X_TCC_DIR must contain libtcc.dll, libtcc/libtcc.h, include and lib",
            ));
        }

        Ok(Some(Self {
            directory,
            object_type,
            ucrt: library::system_crt()?,
        }))
    }
}

pub(super) fn configured(options: &LinkOptions<'_>) -> Result<bool, CompilationFailure> {
    if !supported_options(options) {
        return Ok(false);
    }

    let Some(config) = Configuration::from_environment()? else {
        return Ok(false);
    };

    Ok(config.directory.as_os_str().is_ascii() && config.ucrt.as_os_str().is_ascii())
}

pub(super) fn link_elf(
    objects: &[String],
    output: &str,
    options: &LinkOptions<'_>,
) -> Result<LinkOutcome, CompilationFailure> {
    if !supported_options(options) {
        return Ok(LinkOutcome::Unsupported(
            "optimised runtime, custom linker options or target require Clang",
        ));
    }

    let Some(config) = Configuration::from_environment()? else {
        return Ok(LinkOutcome::Unsupported("X_TCC_DIR is not configured"));
    };

    if !output.is_ascii()
        || objects.iter().any(|object| !object.is_ascii())
        || !config.directory.as_os_str().is_ascii()
        || !config.ucrt.as_os_str().is_ascii()
    {
        return Ok(LinkOutcome::Unsupported(
            "TinyCC narrow filesystem paths require Clang for Unicode names",
        ));
    }

    if objects.is_empty() {
        return Ok(LinkOutcome::Unsupported("no object inputs"));
    }

    for object in objects {
        let bytes = fs::read(object).map_err(failure)?;

        if !cache::elf_object(&bytes) {
            return Ok(LinkOutcome::Unsupported(
                "TinyCC requires x86-64 ELF relocatable objects",
            ));
        }

        if bytes
            .windows(x_runtime::abi::PARALLEL_FOR.len())
            .any(|part| part == x_runtime::abi::PARALLEL_FOR.as_bytes())
        {
            return Ok(LinkOutcome::Unsupported(
                "OpenMP dispatch requires the Clang runtime",
            ));
        }

        if bytes
            .windows(b"__chkstk\0".len())
            .any(|part| part == b"__chkstk\0")
        {
            return Ok(LinkOutcome::Unsupported(
                "LLVM object lacks the TinyCC-specific stack probe",
            ));
        }
    }

    let runtime = PreparedRuntime::new(&config)?;

    if link_objects(&runtime, &config, objects, Path::new(output)).is_err() {
        return Ok(LinkOutcome::Unsupported(
            "TinyCC cannot link this ELF relocation or external symbol set",
        ));
    }

    if options.reproducible && !cache::zero_pe_timestamp(Path::new(output))? {
        return Ok(LinkOutcome::Unsupported(
            "TinyCC output has a nonzero PE timestamp",
        ));
    }

    Ok(LinkOutcome::Linked)
}

fn link_objects(
    runtime: &PreparedRuntime,
    config: &Configuration,
    objects: &[String],
    output: &Path,
) -> Result<(), CompilationFailure> {
    let mut state = runtime.executable()?;

    for object in objects {
        state.add_file(Path::new(object))?;
    }

    prepared::attach_support(&mut state, &config.directory, &config.ucrt)?;

    state.output_file(output)
}

pub(super) fn c_path(path: &Path) -> Result<CString, CompilationFailure> {
    // TinyCC's fopen needs DOS/UNC paths instead of extended-length paths.
    let text = path
        .to_str()
        .ok_or_else(|| failure("TinyCC path is not valid Unicode"))?;
    let text = if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        text.strip_prefix(r"\\?\").unwrap_or(text).to_owned()
    };

    CString::new(text).map_err(failure)
}
