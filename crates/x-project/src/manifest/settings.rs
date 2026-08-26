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

    pub fn parse(value: &str) -> Option<Self> {
        [Self::Binary, Self::Library]
            .into_iter()
            .find(|kind| kind.as_str() == value)
    }
}

#[derive(Debug, Clone)]
pub struct PackageSettings {
    pub name: String,
    pub version: String,
    pub entry: PathBuf,
    pub kind: PackageKind,
}

#[derive(Debug, Clone)]
pub struct DependencySettings {
    pub name: String,
    pub path: PathBuf,
    pub declared_path: String,
}

#[derive(Debug, Clone, Default)]
pub struct LinkSettings {
    pub libraries: Vec<String>,
    pub library_paths: Vec<PathBuf>,
    pub include_paths: Vec<PathBuf>,
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
