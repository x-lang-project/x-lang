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
#[derive(Debug, Clone, Copy)]
pub struct CompileMetrics {
    pub elapsed_seconds: f64,
    pub ir_bytes: usize,
    pub ir_lines: usize,
}
#[derive(Debug, Clone)]
pub struct CompileArtifact {
    pub path: PathBuf,
    pub target_triple: String,
    pub metrics: CompileMetrics,
    pub ir: String,
    pub debug_artifacts: Vec<DebugArtifact>,
}
