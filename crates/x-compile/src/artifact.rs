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

impl CompileArtifact {
    pub(crate) fn new(
        path: PathBuf,
        target_triple: String,
        elapsed_seconds: f64,
        generated: GeneratedIr,
    ) -> Self {
        Self {
            path,
            target_triple,
            metrics: CompileMetrics {
                elapsed_seconds,
                ir_bytes: generated.ir.len(),
                ir_lines: generated.ir.lines().count(),
            },
            ir: generated.ir,
            debug_artifacts: generated.debug_artifacts,
        }
    }
}
