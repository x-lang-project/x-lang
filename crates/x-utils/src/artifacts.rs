use std::ffi;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub const EXECUTABLE_EXTENSION: &str = "exe";
pub const EXECUTABLE_SUFFIX: &str = ".exe";
pub const OBJECT_EXTENSION: &str = "o";
pub const IR_EXTENSION: &str = "ll";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactPlatform {
    Unix,
    Windows,
}

impl ArtifactPlatform {
    pub const fn host() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Unix
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArtifactPaths {
    base: PathBuf,
    platform: ArtifactPlatform,
}

impl ArtifactPaths {
    fn append_extension(&self, extension: &str) -> PathBuf {
        let mut name = self.base.as_os_str().to_os_string();
        name.push(".");
        name.push(extension);

        PathBuf::from(name)
    }
}
