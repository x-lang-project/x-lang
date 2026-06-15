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
    pub fn record(&mut self, artifact: ProofArtifact) {
        self.artifacts.push(artifact);
    }

    pub fn status_for(&self, obligation: &ProofObligation) -> ProofStatus {
        self.recorded_status(obligation)
            .cloned()
            .unwrap_or(ProofStatus::Pending)
    }

    fn recorded_status(&self, obligation: &ProofObligation) -> Option<&ProofStatus> {
        self.artifacts
            .iter()
            .rev()
            .find(|artifact| artifact.obligation_identifier == obligation.identifier)
            .map(|artifact| &artifact.status)
    }

    pub fn registration_report(&self, obligations: &[ProofObligation]) -> RegistrationReport {
        let mut report = RegistrationReport::default();

        for obligation in obligations {
            match self.recorded_status(obligation) {
                None | Some(ProofStatus::Pending) => report.pending += 1,
                Some(ProofStatus::FingerprintMatch) => report.matched += 1,
                Some(ProofStatus::FingerprintMismatch(_)) => report.mismatched += 1,
                Some(ProofStatus::TrustedAssumption(_)) => report.trusted_assumptions += 1,
            }
        }

        report
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RegistrationReport {
    pub pending: usize,
    pub matched: usize,
    pub mismatched: usize,
    pub trusted_assumptions: usize,
}

impl RegistrationReport {
    pub fn is_fully_registered(self) -> bool {
        self.pending == 0 && self.mismatched == 0 && self.trusted_assumptions == 0
    }
}

pub fn match_obligation_fingerprint(
    obligation: &ProofObligation,
    fingerprint: impl AsRef<str>,
) -> ProofArtifact {
    let fingerprint = fingerprint.as_ref();
    let expected = stable_identifier(&obligation.subject, &obligation.predicate);

    ProofArtifact {
        obligation_identifier: obligation.identifier.clone(),
        status: if fingerprint == expected {
            ProofStatus::FingerprintMatch
        } else {
            ProofStatus::FingerprintMismatch("fingerprint does not match obligation".to_string())
        },
        fingerprint: Some(fingerprint.to_string()),
    }
}

pub fn stable_identifier(subject: &str, predicate: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;

    for part in [subject, predicate] {
        for byte in (part.len() as u64)
            .to_le_bytes()
            .into_iter()
            .chain(part.bytes())
        {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }

    format!("{hash:016x}")
}
