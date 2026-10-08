use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::os;
use crate::os::build_os_artifact;
use crate::{BuildArtifact, BuildOutput};
use x_compile::MirObjectOptions;
use x_compile::ModuleCompileOptions;
use x_diagnostics::CompilationFailure;
use x_diagnostics::DriverError;
use x_link::LinkOptions;
use x_mir::MirProgram;
use x_project::ModuleResolver;
use x_project::manifest::lock::ensure_manifest_lock;
use x_project::manifest::read_utf8;
use x_project::manifest::{LinkSettings, Manifest};
use x_utils::artifacts::ArtifactPaths;
use x_utils::target::ProductTarget;

mod modules;
pub use modules::resolve_manifest_modules;

#[derive(Clone, Copy, Debug, Default)]
pub struct ProjectBuildOptions<'a> {
    pub output: Option<&'a Path>,
    pub optimisation_level: Option<u8>,
    pub save_ir: bool,
    pub target: Option<&'a str>,
}

#[derive(Default)]
struct ManifestLinkInputs {
    libs: Vec<String>,
    lib_paths: Vec<String>,
    include_paths: Vec<String>,
}

impl ManifestLinkInputs {
    fn from_settings(settings: &LinkSettings) -> Self {
        Self {
            libs: settings.libraries.clone(),
            lib_paths: settings
                .library_paths
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
            include_paths: settings
                .include_paths
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
        }
    }

    fn options<'a>(&'a self, optimisation_level: u8, target_triple: &'a str) -> LinkOptions<'a> {
        LinkOptions {
            libs: &self.libs,
            lib_paths: &self.lib_paths,
            include_paths: &self.include_paths,
            optimisation_level,
            target_triple: Some(target_triple),
            freestanding: false,
            passthrough_arguments: &[],
            reproducible: true,
            gc_sections: false,
        }
    }
}

mod execution;
pub use execution::*;
