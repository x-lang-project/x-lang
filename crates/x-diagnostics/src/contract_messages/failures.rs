use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ContractValidationPhase {
    #[default]
    Lowering,
    Optimisation,
    ManifestLowering,
}
