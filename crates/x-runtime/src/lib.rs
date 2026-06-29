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

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::source::HOSTED_WITH_PARALLEL as RUNTIME_C;
use x_diagnostics::backend_messages;

pub mod source;

pub use crate::source::ABI_HEADER as RUNTIME_ABI_HEADER;
pub const RUNTIME_ABI_VERSION: u32 = 2;

pub mod abi {
    pub const VERSION: &str = "x_runtime_abi_version";
    pub const ALLOC: &str = "x_runtime_alloc";
    pub const REALLOC: &str = "x_runtime_realloc";
    pub const FREE: &str = "x_runtime_free";
    pub const STRING_LEN: &str = "x_runtime_string_len";
    pub const WRITE_STDOUT: &str = "x_runtime_write_stdout";
    pub const EXIT_CODE: &str = "x_runtime_exit_code";
    pub const PROCESS_EXIT: &str = "x_runtime_process_exit";
    pub const PANIC: &str = "x_runtime_panic";
    pub const PARALLEL_FOR: &str = "dispatch_parallel_for";
    pub const PARALLEL_FOR_CONTEXT: &str = "dispatch_parallel_for_context";

    pub const EXPORTED_FUNCTIONS: [&str; 11] = [
        VERSION,
        ALLOC,
        REALLOC,
        FREE,
        STRING_LEN,
        WRITE_STDOUT,
        EXIT_CODE,
        PROCESS_EXIT,
        PANIC,
        PARALLEL_FOR,
        PARALLEL_FOR_CONTEXT,
    ];
}
static NEXT_RUNTIME_FILE: AtomicU64 = AtomicU64::new(0);

pub fn get_runtime_c_blob() -> String {
    RUNTIME_C.to_owned()
}

pub fn write_runtime_file(
    output_directory: &Path,
) -> Result<String, backend_messages::RuntimeSourceError> {
    let sequence = NEXT_RUNTIME_FILE.fetch_add(1, Ordering::Relaxed);
    let c_path = output_directory.join(format!(".x-runtime-{}-{sequence}.c", process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&c_path)
        .map_err(|error| backend_messages::RuntimeSourceError::Create {
            path: c_path.clone(),
            cause: error,
        })?;

    if let Err(error) = file.write_all(RUNTIME_C.as_bytes()) {
        // Close the handle before removing the partial file on Windows.
        drop(file);
        let _ = fs::remove_file(&c_path);

        return Err(backend_messages::RuntimeSourceError::Write {
            path: c_path.clone(),
            cause: error,
        });
    }

    Ok(c_path.to_string_lossy().into_owned())
}
