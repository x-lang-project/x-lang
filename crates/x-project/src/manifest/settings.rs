use std::path::{Path, PathBuf};

use super::{manifest_string_array, required_manifest_string};
use decode::*;
use x_diagnostics::CompilationFailure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageKind {
    Binary,
    Library,
}

impl PackageKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Binary => "bin",
            Self::Library => "lib",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DependencySettings {
    pub name: String,
    pub path: PathBuf,
    pub declared_path: String,
}

#[derive(Debug, Clone)]
pub struct OsBuildSettings {
    pub boot_asm: PathBuf,
    pub crt0: PathBuf,
    pub support: Vec<PathBuf>,
    pub linker_script: PathBuf,
    pub kernel: PathBuf,
    pub iso: Option<PathBuf>,
    pub iso_directory: Option<PathBuf>,
    pub grub_cfg: Option<PathBuf>,
    pub nasm: String,
    pub cc: String,
    pub cxx: String,
    pub ld: String,
    pub grub_mkrescue: Option<String>,
}

mod decode;
