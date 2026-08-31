use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::llvm_policy;
use super::llvm_toolchain::llvm_prefix;

const FILE_COMPARISON_CHUNK_BYTES: usize = 64 * 1024;

pub(crate) fn stage_windows_llvm_runtime_next_to_binary() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let prefix = llvm_prefix().unwrap_or_else(|| {
        panic!(
            "LLVM {} not found; set {} to its installation directory",
            llvm_policy::LLVM_VERSION,
            llvm_policy::LLVM_PREFIX_ENV
        )
    });
    let clang = prefix.join("bin").join("clang.exe");
    let version = Command::new(&clang)
        .arg("--version")
        .output()
        .unwrap_or_else(|error| panic!("failed to query {}: {error}", clang.display()));
    let version_text = String::from_utf8_lossy(&version.stdout);
    assert!(
        llvm_policy::clang_is_pinned(&version_text),
        "LLVM at {} does not match version {}: {}",
        prefix.display(),
        llvm_policy::LLVM_VERSION,
        version_text.trim()
    );

    let source = prefix.join("bin").join("LLVM-C.dll");
    let output_directory = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .expect("OUT_DIR is not set");
    let profile_directory = output_directory
        .ancestors()
        .nth(3)
        .expect("OUT_DIR must be inside target/<profile>/build/<crate>/out");
    let destination = profile_directory.join("LLVM-C.dll");
    println!("cargo:rerun-if-changed={}", source.display());
    if identical_files(&source, &destination).unwrap_or(false) {
        return;
    }

    fs::copy(&source, &destination).unwrap_or_else(|error| {
        panic!(
            "failed to copy {} to {}: {error}",
            source.display(),
            destination.display()
        )
    });
}

// Do not overwrite a DLL that may already be loaded.
pub(crate) fn identical_files(source: &Path, destination: &Path) -> io::Result<bool> {
    use std::io::Read;

    let mut source = File::open(source)?;
    let mut destination = File::open(destination)?;

    if source.metadata()?.len() != destination.metadata()?.len() {
        return Ok(false);
    }

    let mut source_buffer = [0_u8; FILE_COMPARISON_CHUNK_BYTES];
    let mut destination_buffer = [0_u8; FILE_COMPARISON_CHUNK_BYTES];

    loop {
        let count = source.read(&mut source_buffer)?;

        if count == 0 {
            return Ok(true);
        }

        destination.read_exact(&mut destination_buffer[..count])?;

        if source_buffer[..count] != destination_buffer[..count] {
            return Ok(false);
        }
    }
}
