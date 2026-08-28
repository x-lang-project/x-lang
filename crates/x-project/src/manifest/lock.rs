use std::fmt::Write;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::LANGUAGE_EDITION;
use super::read_utf8;
use super::required_manifest_string;
use super::resolve_manifest_relative_path;
use x_diagnostics::{CompilationFailure, ProjectError};
use x_utils::project::LOCK_FILE_NAME;
use x_utils::{project::manifest_in as first_existing_manifest, temporary};

const LOCK_FORMAT_VERSION: u32 = 1;

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);

    for byte in digest {
        let _ = write!(hex, "{byte:02x}");
    }

    hex
}

fn toml_string(value: &str) -> String {
    toml::Value::String(value.to_string()).to_string()
}
