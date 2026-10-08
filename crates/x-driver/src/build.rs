use super::*;
use x_compile::CompilationFailure;
use x_compile::CompileArtifact;
use x_compile::IrCompileOptions;
use x_compile::ObjectCompileOptions;
use x_diagnostics::DriverError;
use x_utils::artifacts::ArtifactPaths;

#[derive(Debug)]
pub struct BuildArtifact {
    pub path: PathBuf,
    pub compilation: CompileArtifact,
    pub executable: bool,
}

#[derive(Debug)]
pub enum BuildOutput {
    Native(BuildArtifact),
    C(CBuildArtifact),
}

impl BuildOutput {
    pub fn path(&self) -> &Path {
        match self {
            Self::Native(artifact) => &artifact.path,
            Self::C(artifact) => &artifact.path,
        }
    }

    pub fn executable(&self) -> bool {
        match self {
            Self::Native(artifact) => artifact.executable,
            Self::C(_) => true,
        }
    }
}

#[derive(Debug)]
pub struct CBuildArtifact {
    pub path: PathBuf,
    pub target_triple: String,
    pub compilation: x_compile::CArtifact,
    pub debug_artifacts: Vec<x_compile::DebugArtifact>,
    pub elapsed_seconds: f64,
}
