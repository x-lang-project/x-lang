use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DebugArtifact {
    pub stage: &'static str,
    pub text: String,
}
