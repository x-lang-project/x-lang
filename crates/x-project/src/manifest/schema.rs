use std::path::Component;
use std::path::Path;

use super::manifest_string_array;
use super::{LANGUAGE_EDITION, RUNTIME_ABI_VERSION};
use freestanding::{validate_build, validate_os, validate_qemu};
use target::*;
use x_diagnostics::CompilationFailure;
use x_diagnostics::ProjectError;
use x_utils::optimization::NativeOptimization;

pub fn validate_manifest_schema(
    path: &Path,
    value: &toml::Value,
) -> Result<(), CompilationFailure> {
    let root = value
        .as_table()
        .ok_or_else(|| ProjectError::manifest_tables_required(path.display()))?;
    let is_os_manifest = root.get("os").is_some();
    let allowed_root = if is_os_manifest {
        &[
            "package",
            "workspace",
            "toolchain",
            "dependencies",
            "link",
            "target",
            "build",
            "os",
            "qemu",
        ][..]
    } else {
        &[
            "package",
            "workspace",
            "toolchain",
            "dependencies",
            "link",
            "target",
        ][..]
    };

    reject_unknown_keys(path, "root", root, allowed_root)?;

    let package = root.get("package").and_then(toml::Value::as_table);
    let workspace = root.get("workspace").and_then(toml::Value::as_table);

    if package.is_none() && workspace.is_none() {
        return Err(ProjectError::package_or_workspace_missing(path.display()));
    }

    if let Some(package) = package {
        validate_package(path, package)?;
        validate_toolchain(path, root)?;
    }

    if let Some(workspace) = workspace {
        validate_workspace(path, workspace)?;
    }

    if let Some(dependencies) = root.get("dependencies") {
        validate_dependencies(path, dependencies)?;
    }

    if let Some(link) = root.get("link") {
        validate_link(path, link)?;
    }

    if is_os_manifest && root.get("build").is_none() {
        return Err(ProjectError::os_build_section_required(path.display()));
    }

    if let Some(build) = root.get("build") {
        validate_build(path, build)?;
    }

    if let Some(os) = root.get("os") {
        validate_os(path, os)?;
    }

    if let Some(qemu) = root.get("qemu") {
        validate_qemu(path, qemu)?;
    }

    if let Some(target) = root.get("target") {
        validate_target(path, target, is_os_manifest)?;
    }

    Ok(())
}

fn validate_package(path: &Path, package: &toml::value::Table) -> Result<(), CompilationFailure> {
    reject_unknown_keys(
        path,
        "package",
        package,
        &["name", "version", "edition", "entry", "kind"],
    )?;
    let name = required_manifest_string(path, "package", package, "name")?;

    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(ProjectError::package_name(path.display()));
    }

    let version = required_manifest_string(path, "package", package, "version")?;

    if !is_canonical_version(version) {
        return Err(ProjectError::package_version(path.display()));
    }

    let edition = required_manifest_string(path, "package", package, "edition")?;

    if edition != LANGUAGE_EDITION {
        return Err(ProjectError::language_edition(
            path.display(),
            edition,
            LANGUAGE_EDITION,
        ));
    }

    let entry = required_manifest_string(path, "package", package, "entry")?;
    let entry_path = Path::new(entry);

    if entry_path.is_absolute()
        || entry_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ProjectError::package_entry_path(path.display(), entry));
    }

    if let Some(kind) = package.get("kind") {
        let kind = kind
            .as_str()
            .ok_or_else(|| ProjectError::package_kind_type(path.display()))?;

        if super::PackageKind::parse(kind).is_none() {
            return Err(ProjectError::package_kind_unsupported(path.display(), kind));
        }
    }

    Ok(())
}

fn validate_workspace(
    path: &Path,
    workspace: &toml::value::Table,
) -> Result<(), CompilationFailure> {
    reject_unknown_keys(path, "workspace", workspace, &["members", "default_member"])?;
    validate_string_array(path, "workspace.members", workspace.get("members"), true)?;
    if let Some(default_member) = workspace.get("default_member")
        && default_member.as_str().is_none()
    {
        return Err(ProjectError::workspace_default_member(path.display()));
    }

    Ok(())
}

fn validate_dependencies(
    path: &Path,
    dependencies: &toml::Value,
) -> Result<(), CompilationFailure> {
    let dependencies = dependencies
        .as_table()
        .ok_or_else(|| ProjectError::dependencies_table(path.display()))?;

    for (name, specification) in dependencies {
        let specification = specification
            .as_table()
            .ok_or_else(|| ProjectError::dependency_path_table(path.display(), name))?;
        reject_unknown_keys(
            path,
            &format!("dependencies.{name}"),
            specification,
            &["path"],
        )?;
        required_manifest_string(path, &format!("dependencies.{name}"), specification, "path")?;
    }

    Ok(())
}

fn validate_link(path: &Path, link: &toml::Value) -> Result<(), CompilationFailure> {
    let link = link
        .as_table()
        .ok_or_else(|| ProjectError::link_table(path.display()))?;
    reject_unknown_keys(path, "link", link, &["libs", "lib_paths", "include_paths"])?;
    for key in ["libs", "lib_paths", "include_paths"] {
        validate_string_array(path, &format!("link.{key}"), link.get(key), false)?;
    }

    for library in manifest_string_array(link, "libs") {
        if library.is_empty()
            || library.starts_with('-')
            || !library
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        {
            return Err(ProjectError::link_library_name(path.display(), library));
        }
    }

    for key in ["lib_paths", "include_paths"] {
        for value in manifest_string_array(link, key) {
            if !is_portable_package_relative_path(&value) {
                return Err(ProjectError::link_path_escapes_package(
                    path.display(),
                    key,
                    value,
                ));
            }
        }
    }

    Ok(())
}

mod freestanding;

mod values;
pub use values::*;

mod target;
