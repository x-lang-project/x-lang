use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DebugArtifact {
    pub stage: &'static str,
    pub text: String,
}
#[derive(Debug, Clone)]
pub struct GeneratedIr {
    pub ir: String,
    pub debug_artifacts: Vec<DebugArtifact>,
}
