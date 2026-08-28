use std::path::Component;
use std::path::Path;

use super::manifest_string_array;

use freestanding::{validate_build, validate_os, validate_qemu};
use target::*;
use x_diagnostics::CompilationFailure;
use x_diagnostics::ProjectError;
use x_utils::optimization::NativeOptimization;

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
