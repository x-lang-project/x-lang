#[derive(Debug, Clone)]
pub struct CEmissionMetrics {
    pub elapsed_seconds: f64,
    pub source_bytes: usize,
    pub source_lines: usize,
}

#[derive(Debug, Clone)]
pub struct CArtifact {
    pub source: String,
    pub metrics: CEmissionMetrics,
}
