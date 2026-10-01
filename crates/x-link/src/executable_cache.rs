use std::{
    cell::RefCell,
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::NativeToolchain;

#[must_use]
pub struct Capture;

pub fn toolchain_identity() -> Option<Vec<u8>> {
    if !cfg!(all(windows, target_arch = "x86_64")) || env::var_os("X_CLANG").is_some() {
        return None;
    }

    let toolchain = NativeToolchain::discover().ok()?;
    let compiler = toolchain.compiler().canonicalize().ok()?;
    let directory = compiler.parent()?;
    let mut identity = NativeToolchain::discovery_fingerprint()?;
    let executable = env::current_exe().ok()?;
    let loaded_llvm = executable.parent()?.join("LLVM-C.dll");

    for path in [
        compiler.clone(),
        directory.join("lld-link.exe"),
        directory.join("LLVM-C.dll"),
        loaded_llvm,
        executable,
    ] {
        identity.extend(NativeToolchain::file_identity(&path)?);
    }

    identity.extend(toolchain.cache_identity()?);

    Some(identity)
}
