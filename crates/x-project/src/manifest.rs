use std::path::{Path, PathBuf};

use x_diagnostics::CompilationFailure;
use x_diagnostics::ManifestPosition;
use x_diagnostics::ProjectError;

mod schema;
mod settings;
pub use settings::{
    BuildSettings, DependencySettings, EmulatorSettings, LinkSettings, OsBuildSettings,
    PackageKind, PackageSettings, WorkspaceSettings,
};

pub use schema::is_canonical_version;
pub use schema::required_manifest_string;

pub use x_runtime::RUNTIME_ABI_VERSION;

#[derive(Debug, Clone)]
pub struct Manifest {
    path: PathBuf,
    source: String,
    value: toml::Value,
    settings: settings::ManifestSettings,
}
impl Manifest {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn target(&self) -> Option<&str> {
        self.settings.target.as_deref()
    }

    pub fn package(&self) -> Option<&PackageSettings> {
        self.settings.package.as_ref()
    }

    pub fn workspace(&self) -> Option<&WorkspaceSettings> {
        self.settings.workspace.as_ref()
    }

    pub fn dependencies(&self) -> &[DependencySettings] {
        &self.settings.dependencies
    }
}

pub use crate::source::read_utf8;

pub fn manifest_string_array(table: &toml::value::Table, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}
