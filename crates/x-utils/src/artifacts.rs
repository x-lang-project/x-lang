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
    pub fn executable(&self) -> PathBuf {
        let already_executable = self
            .base
            .as_os_str()
            .to_string_lossy()
            .to_ascii_lowercase()
            .ends_with(EXECUTABLE_SUFFIX);

        if self.platform == ArtifactPlatform::Windows && !already_executable {
            let mut name = self.base.as_os_str().to_os_string();
            name.push(EXECUTABLE_SUFFIX);

            PathBuf::from(name)
        } else {
            self.base.clone()
        }
    }

    /// An explicit output extension belongs to the caller.
    pub fn executable_override(&self) -> PathBuf {
        if self.platform == ArtifactPlatform::Windows && self.base.extension().is_none() {
            self.base.with_extension(EXECUTABLE_EXTENSION)
        } else {
            self.base.clone()
        }
    }

    pub fn object(&self) -> PathBuf {
        self.base.with_extension(OBJECT_EXTENSION)
    }

    pub fn ir(&self) -> PathBuf {
        self.base.with_extension(IR_EXTENSION)
    }

    /// A generated module name is complete, even when it contains dots.
    pub fn generated_object(&self) -> PathBuf {
        self.append_extension(OBJECT_EXTENSION)
    }

    pub fn generated_ir(&self) -> PathBuf {
        self.append_extension(IR_EXTENSION)
    }

    fn append_extension(&self, extension: &str) -> PathBuf {
        let mut name = self.base.as_os_str().to_os_string();
        name.push(".");
        name.push(extension);

        PathBuf::from(name)
    }

    pub fn archived_ir(&self, optimisation: u8) -> PathBuf {
        let mut name = OsString::from(
            self.base
                .file_stem()
                .unwrap_or_else(|| ffi::OsStr::new("output")),
        );
        name.push(format!("_0{optimisation}.{IR_EXTENSION}"));

        self.base.parent().unwrap_or(Path::new(".")).join(name)
    }
}
