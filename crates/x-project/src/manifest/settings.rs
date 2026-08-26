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
pub struct WorkspaceSettings {
    pub members: Vec<String>,
    pub default_member: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DependencySettings {
    pub name: String,
    pub path: PathBuf,
    pub declared_path: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildSettings {
    pub optimization: Option<u8>,
    pub no_borrow_check: bool,
    pub freestanding: bool,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmulatorSettings {
    pub command: String,
    pub memory: String,
}

impl Default for EmulatorSettings {
    fn default() -> Self {
        Self {
            command: "qemu-system-i386".into(),
            memory: "512M".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ManifestSettings {
    pub package: Option<PackageSettings>,
    pub workspace: Option<WorkspaceSettings>,
    pub dependencies: Vec<DependencySettings>,
    pub build: BuildSettings,
    pub link: LinkSettings,
    pub os: Option<OsBuildSettings>,
    pub emulator: EmulatorSettings,
    pub target: Option<String>,
}

impl ManifestSettings {
    pub fn decode(path: &Path, value: &toml::Value) -> Result<Self, CompilationFailure> {
        let root = path.parent().unwrap_or(Path::new("."));
        let package = value
            .get("package")
            .and_then(toml::Value::as_table)
            .map(|table| decode_package(path, root, table))
            .transpose()?;

        let build = value.get("build");
        let build = BuildSettings {
            optimization: build
                .and_then(|build| build.get("optimization"))
                .and_then(toml::Value::as_integer)
                .and_then(|level| u8::try_from(level).ok()),
            no_borrow_check: boolean(build, "no_borrow_check"),
            freestanding: boolean(build, "freestanding"),
        };

        let link = decode_link(root, value.get("link").and_then(toml::Value::as_table));
        let os = value
            .get("os")
            .and_then(toml::Value::as_table)
            .map(|table| decode_os(path, root, table))
            .transpose()?;

        let defaults = EmulatorSettings::default();
        let qemu = value.get("qemu");
        let emulator = EmulatorSettings {
            command: text(qemu, "command").unwrap_or(&defaults.command).into(),
            memory: text(qemu, "memory").unwrap_or(&defaults.memory).into(),
        };

        Ok(Self {
            package,
            workspace: value
                .get("workspace")
                .and_then(toml::Value::as_table)
                .map(|table| WorkspaceSettings {
                    members: manifest_string_array(table, "members"),
                    default_member: table
                        .get("default_member")
                        .and_then(toml::Value::as_str)
                        .map(String::from),
                }),
            dependencies: decode_dependencies(root, value.get("dependencies"))?,
            build,
            link,
            os,
            emulator,
            target: text(value.get("target"), "triple").map(String::from),
        })
    }
}

fn boolean(section: Option<&toml::Value>, key: &str) -> bool {
    section
        .and_then(|section| section.get(key))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn text<'a>(section: Option<&'a toml::Value>, key: &str) -> Option<&'a str> {
    section?.get(key)?.as_str()
}

mod decode;
