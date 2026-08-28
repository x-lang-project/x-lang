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

#[derive(Debug, Clone)]
pub struct ManifestCrate {
    pub import_name: String,
    pub entry_path: PathBuf,
}

pub(super) fn default_member_manifest(
    workspace_manifest: &Path,
    workspace: Option<&super::WorkspaceSettings>,
) -> Result<PathBuf, CompilationFailure> {
    let workspace =
        workspace.ok_or_else(|| ProjectError::workspace_missing(workspace_manifest.display()))?;

    if workspace.members.is_empty() {
        return Err(ProjectError::workspace_members_empty(
            workspace_manifest.display(),
        ));
    }

    let selected = workspace
        .default_member
        .as_deref()
        .or_else(|| workspace.members.first().map(String::as_str))
        .ok_or_else(|| ProjectError::workspace_selection_missing(workspace_manifest.display()))?;
    let root = workspace_manifest.parent().unwrap_or(Path::new("."));

    member_manifest_path(root, selected)
}

fn register_manifest_crate(
    crates: &mut BTreeMap<String, PathBuf>,
    import_name: String,
    entry_path: PathBuf,
) -> Result<(), CompilationFailure> {
    if !entry_path.is_file() {
        return Err(ProjectError::package_entry_missing(
            entry_path.display(),
            import_name,
        ));
    }

    let canonical = entry_path
        .canonicalize()
        .map_err(|error| ProjectError::package_entry_resolution(entry_path.display(), error))?;

    if let Some(existing) = crates.get(&import_name) {
        if existing == &canonical {
            return Ok(());
        }

        return Err(ProjectError::package_import_ambiguous(
            existing.display(),
            canonical.display(),
            import_name,
        ));
    }

    crates.insert(import_name, canonical);

    Ok(())
}

fn member_manifest_path(
    workspace_root: &Path,
    member: &str,
) -> Result<PathBuf, CompilationFailure> {
    let member_root = resolve_manifest_relative_path(workspace_root, member);

    first_existing_manifest(&member_root).ok_or_else(|| {
        ProjectError::workspace_member_manifest_missing(member_root.display(), member)
    })
}
