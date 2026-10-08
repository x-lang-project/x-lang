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
