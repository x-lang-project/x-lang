use std::fmt::Write;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::{
    LANGUAGE_EDITION, RUNTIME_ABI_VERSION, read_utf8, required_manifest_string,
    resolve_manifest_relative_path, validate_manifest_schema,
};
use x_diagnostics::{CompilationFailure, ProjectError};
use x_utils::project::LOCK_FILE_NAME;
use x_utils::{project::manifest_in as first_existing_manifest, temporary};

const LOCK_FORMAT_VERSION: u32 = 1;

pub fn ensure_manifest_lock(
    manifest_path: &Path,
    manifest_source: &str,
    value: &toml::Value,
) -> Result<(), CompilationFailure> {
    if value
        .get("package")
        .and_then(toml::Value::as_table)
        .is_none()
    {
        return Ok(());
    }

    let expected = render_manifest_lock(manifest_path, manifest_source, value)?;
    let root = manifest_path.parent().unwrap_or(Path::new("."));
    let lock_path = root.join(LOCK_FILE_NAME);

    if lock_path.exists() {
        let actual = read_utf8(&lock_path)?;

        if actual == expected {
            return Ok(());
        }

        return Err(ProjectError::lock_stale(lock_path.display()));
    }

    let mut candidate = temporary::TemporaryFile::create(root, LOCK_FILE_NAME)
        .map_err(|error| ProjectError::lock_candidate_create(root.display(), error))?;
    candidate
        .write_and_sync(expected.as_bytes())
        .map_err(|error| ProjectError::lock_candidate_write(candidate.path().display(), error))?;

    candidate
        .install(&lock_path)
        .map_err(|error| ProjectError::lock_install(lock_path.display(), error))
}

fn render_manifest_lock(
    manifest_path: &Path,
    manifest_source: &str,
    value: &toml::Value,
) -> Result<String, CompilationFailure> {
    let root = manifest_path.parent().unwrap_or(Path::new("."));
    let package = value
        .get("package")
        .and_then(toml::Value::as_table)
        .ok_or_else(ProjectError::lock_package_missing)?;
    let name = required_manifest_string(manifest_path, "package", package, "name")?;
    let version = required_manifest_string(manifest_path, "package", package, "version")?;
    let mut output = format!(
        "lock-version = {LOCK_FORMAT_VERSION}\nlanguage-edition = {}\nruntime-abi = {RUNTIME_ABI_VERSION}\n\n[package]\nname = {}\nversion = {}\nmanifest-sha256 = {}\n",
        toml_string(LANGUAGE_EDITION),
        toml_string(name),
        toml_string(version),
        toml_string(&sha256_hex(manifest_source.as_bytes())),
    );

    if let Some(dependencies) = value.get("dependencies").and_then(toml::Value::as_table) {
        for (dependency_name, specification) in dependencies {
            let specification = specification
                .as_table()
                .ok_or_else(|| ProjectError::lock_dependency_shape(dependency_name))?;
            let dependency_path = required_manifest_string(
                manifest_path,
                &format!("dependencies.{dependency_name}"),
                specification,
                "path",
            )?;
            let dependency_root = resolve_manifest_relative_path(root, dependency_path);
            let dependency_manifest =
                first_existing_manifest(&dependency_root).ok_or_else(|| {
                    ProjectError::lock_dependency_manifest_missing(
                        dependency_root.display(),
                        dependency_name,
                    )
                })?;
            let dependency_source = read_utf8(&dependency_manifest)?;
            let dependency_value =
                toml::from_str::<toml::Value>(&dependency_source).map_err(|error| {
                    ProjectError::lock_dependency_parse(dependency_manifest.display(), error)
                })?;
            validate_manifest_schema(&dependency_manifest, &dependency_value)?;
            output.push_str(&format!(
                "\n[[dependency]]\nname = {}\npath = {}\nmanifest-sha256 = {}\n",
                toml_string(dependency_name),
                toml_string(dependency_path),
                toml_string(&sha256_hex(dependency_source.as_bytes())),
            ));
        }
    }

    Ok(output)
}

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
