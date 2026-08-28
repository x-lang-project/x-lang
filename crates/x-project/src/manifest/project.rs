use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::Manifest;
use super::resolve_manifest_relative_path;
use x_diagnostics::CompilationFailure;
use x_diagnostics::ProjectError;
use x_utils::project::MANIFEST_FILE_NAME;
use x_utils::{
    project::manifest_in as first_existing_manifest, sanitize_symbol as crate_import_name,
};

fn member_manifest_path(
    workspace_root: &Path,
    member: &str,
) -> Result<PathBuf, CompilationFailure> {
    let member_root = resolve_manifest_relative_path(workspace_root, member);

    first_existing_manifest(&member_root).ok_or_else(|| {
        ProjectError::workspace_member_manifest_missing(member_root.display(), member)
    })
}
