use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use x_diagnostics::CompilationFailure;
use x_diagnostics::link_messages;
use x_utils::tools::LLVM_VERSION as REQUIRED_CLANG_VERSION;
use x_utils::tools::{LLVM_PREFIX_ENV, clang_is_pinned};

#[derive(Debug, Clone)]
pub struct NativeToolchain {
    compiler: PathBuf,
    version: String,
}

impl NativeToolchain {
    pub fn discover() -> Result<Self, CompilationFailure> {
        discover_clang()
    }

    pub fn compiler(&self) -> &Path {
        &self.compiler
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub(crate) fn discovery_fingerprint() -> Option<Vec<u8>> {
        discovery_state::fingerprint()
    }

    pub(crate) fn file_identity(path: &Path) -> Option<Vec<u8>> {
        version_cache::file_identity_bytes(path)
    }

    pub(crate) fn tool_identity(path: &Path) -> Option<Vec<u8>> {
        version_cache::identity_bytes(path)
    }

    pub(crate) fn cache_identity(&self) -> Option<Vec<u8>> {
        version_cache::identity_bytes(&self.compiler)
    }

    pub fn command(&self) -> Command {
        Command::new(&self.compiler)
    }
}

mod discovery_state;
mod version_cache;

pub fn clang_program() -> Result<PathBuf, CompilationFailure> {
    NativeToolchain::discover().map(|toolchain| toolchain.compiler)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigurationSource {
    ExplicitCompiler,
    LlvmPrefix,
}

impl ConfigurationSource {
    const fn environment_name(self) -> &'static str {
        match self {
            Self::ExplicitCompiler => "X_CLANG",
            Self::LlvmPrefix => LLVM_PREFIX_ENV,
        }
    }

    fn version(self, candidate: &Path) -> Option<String> {
        match self {
            Self::ExplicitCompiler => query_clang_version(candidate),
            Self::LlvmPrefix => clang_version(candidate),
        }
    }
}

fn discover_clang() -> Result<NativeToolchain, CompilationFailure> {
    if let Some(explicit) = env::var_os(ConfigurationSource::ExplicitCompiler.environment_name()) {
        return require_llvm_22_clang(
            PathBuf::from(explicit),
            ConfigurationSource::ExplicitCompiler,
        );
    }

    if let Some(prefix) = env::var_os(ConfigurationSource::LlvmPrefix.environment_name()) {
        return require_llvm_22_clang(
            PathBuf::from(prefix).join("bin").join(clang_name()),
            ConfigurationSource::LlvmPrefix,
        );
    }

    let mut candidates = Vec::new();
    #[cfg(windows)]
    if let Some(profile) = env::var_os("USERPROFILE") {
        candidates.push(
            PathBuf::from(profile)
                .join("scoop/apps/llvm/current/bin")
                .join(clang_name()),
        );
    }
    #[cfg(windows)]
    if let Some(program_files) = env::var_os("ProgramFiles") {
        candidates.push(
            PathBuf::from(program_files)
                .join("LLVM/bin")
                .join(clang_name()),
        );
    }
    #[cfg(windows)]
    if let Some(system_drive) = env::var_os("SystemDrive") {
        let root = PathBuf::from(format!("{}\\", system_drive.to_string_lossy()));
        candidates.push(root.join("LLVM/bin").join(clang_name()));
    }

    let major = REQUIRED_CLANG_VERSION
        .split('.')
        .next()
        .unwrap_or(REQUIRED_CLANG_VERSION);
    candidates.extend([
        PathBuf::from(format!("clang-{major}")),
        PathBuf::from("clang"),
    ]);

    for candidate in candidates {
        if let Some(version) = clang_version(&candidate).filter(|version| clang_is_pinned(version))
        {
            return Ok(NativeToolchain {
                compiler: candidate,
                version,
            });
        }
    }

    Err(link_messages::clang_missing(
        REQUIRED_CLANG_VERSION,
        LLVM_PREFIX_ENV,
    ))
}

fn require_llvm_22_clang(
    candidate: PathBuf,
    source: ConfigurationSource,
) -> Result<NativeToolchain, CompilationFailure> {
    let configured_by = source.environment_name();
    let version = source.version(&candidate).ok_or_else(|| {
        link_messages::configured_clang_unavailable(configured_by, candidate.display())
    })?;

    if clang_is_pinned(&version) {
        Ok(NativeToolchain {
            compiler: candidate,
            version,
        })
    } else {
        Err(link_messages::configured_clang_version(
            configured_by,
            candidate.display(),
            REQUIRED_CLANG_VERSION,
            version.lines().next().unwrap_or("unknown version"),
        ))
    }
}

fn clang_version(candidate: &Path) -> Option<String> {
    version_cache::version(candidate, || query_clang_version(candidate))
}

fn query_clang_version(candidate: &Path) -> Option<String> {
    let output = Command::new(candidate).arg("--version").output().ok()?;

    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

fn clang_name() -> &'static str {
    if cfg!(windows) { "clang.exe" } else { "clang" }
}
