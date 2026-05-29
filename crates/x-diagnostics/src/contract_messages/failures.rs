use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ContractValidationPhase {
    #[default]
    Lowering,
    Optimisation,
    ManifestLowering,
}

impl ContractValidationPhase {
    pub fn note(self) -> &'static str {
        match self {
            Self::Lowering => "contract validation failed before lowering",
            Self::Optimisation => "optimiser contract validation failed before optimisation",
            Self::ManifestLowering => "contract validation failed before manifest lowering",
        }
    }
}
