use std::path::{Path, PathBuf};

use x_diagnostics::CompilationFailure;
use x_diagnostics::ManifestPosition;
use x_diagnostics::ProjectError;

mod schema;
mod settings;
pub use settings::DependencySettings;
pub use settings::LinkSettings;
pub use settings::OsBuildSettings;
pub use settings::PackageKind;
pub use settings::PackageSettings;

pub use schema::required_manifest_string;

pub use x_runtime::RUNTIME_ABI_VERSION;

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
