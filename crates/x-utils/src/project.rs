use std::path::{Path, PathBuf};

pub const MANIFEST_FILE_NAME: &str = "x.toml";
pub const LOCK_FILE_NAME: &str = "x.lock";

pub fn manifest_in(directory: &Path) -> Option<PathBuf> {
    let path = directory.join(MANIFEST_FILE_NAME);

    path.is_file().then_some(path)
}

pub fn find_project_manifest(start: &Path) -> Option<PathBuf> {
    start.ancestors().find_map(manifest_in)
}
