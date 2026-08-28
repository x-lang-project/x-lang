use std::path::{Path, PathBuf};

use x_diagnostics::CompilationFailure;
use x_diagnostics::ManifestPosition;
use x_diagnostics::ProjectError;

pub mod lock;
pub mod project;
mod schema;
mod settings;
pub use settings::{
    BuildSettings, DependencySettings, EmulatorSettings, LinkSettings, OsBuildSettings,
    PackageKind, PackageSettings, WorkspaceSettings,
};

pub use schema::{is_canonical_version, required_manifest_string, validate_manifest_schema};
pub const LANGUAGE_EDITION: &str = "2026";
pub use x_runtime::RUNTIME_ABI_VERSION;

#[derive(Debug, Clone)]
pub struct Manifest {
    path: PathBuf,
    source: String,
    value: toml::Value,
    settings: settings::ManifestSettings,
}
impl Manifest {
    pub fn load(path: &Path) -> Result<Self, CompilationFailure> {
        let source = read_utf8(path)?;

        Self::parse(path, &source)
    }

    pub fn parse(path: &Path, source: &str) -> Result<Self, CompilationFailure> {
        let value = toml::from_str(source)
            .map_err(|error| render_manifest_toml_error(path, source, error))?;

        validate_manifest_schema(path, &value)?;
        let settings = settings::ManifestSettings::decode(path, &value)?;

        Ok(Self {
            path: path.into(),
            source: source.into(),
            value,
            settings,
        })
    }

    pub fn value(&self) -> &toml::Value {
        &self.value
    }

    pub fn into_value(self) -> toml::Value {
        self.value
    }

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

    pub fn default_member_manifest(&self) -> Result<PathBuf, CompilationFailure> {
        project::default_member_manifest(&self.path, self.workspace())
    }

    pub fn discover_crates(
        &self,
        current_package_name: &str,
    ) -> Result<Vec<project::ManifestCrate>, CompilationFailure> {
        project::discover_crates(&self.path, self.dependencies(), current_package_name)
    }

    pub fn build_options(&self) -> BuildSettings {
        self.settings.build
    }

    pub fn link_options(&self) -> &LinkSettings {
        &self.settings.link
    }

    pub fn os_options(&self) -> Option<&OsBuildSettings> {
        self.settings.os.as_ref()
    }

    pub fn emulator_options(&self) -> &EmulatorSettings {
        &self.settings.emulator
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

pub fn render_manifest_toml_error(
    path: &Path,
    source: &str,
    error: toml::de::Error,
) -> CompilationFailure {
    let mut byte = error
        .span()
        .map(|span| span.start)
        .unwrap_or(0)
        .min(source.len());

    while byte > 0 && !source.is_char_boundary(byte) {
        byte = byte.saturating_sub(1);
    }

    let prefix = source.get(..byte).unwrap_or("");
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit('\n')
        .next()
        .map(|line| line.chars().count() + 1)
        .unwrap_or(1);

    ProjectError::manifest_toml_parse(
        path.display(),
        ManifestPosition { line, column, byte },
        error,
    )
}

pub(super) fn resolve_manifest_relative_path(root: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}
