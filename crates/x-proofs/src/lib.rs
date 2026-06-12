//! Proof obligation records and fingerprint matching.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProofObligation {
    pub identifier: String,
    pub subject: String,
    pub predicate: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofStatus {
    Pending,
    FingerprintMatch,
    FingerprintMismatch(String),
    TrustedAssumption(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofArtifact {
    pub obligation_identifier: String,
    pub status: ProofStatus,
    pub fingerprint: Option<String>,
}

#[derive(Debug, Default)]
pub struct ProofRegistry {
    artifacts: Vec<ProofArtifact>,
}

impl ProofRegistry {
    fn recorded_status(&self, obligation: &ProofObligation) -> Option<&ProofStatus> {
        self.artifacts
            .iter()
            .rev()
            .find(|artifact| artifact.obligation_identifier == obligation.identifier)
            .map(|artifact| &artifact.status)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RegistrationReport {
    pub pending: usize,
    pub matched: usize,
    pub mismatched: usize,
    pub trusted_assumptions: usize,
}
