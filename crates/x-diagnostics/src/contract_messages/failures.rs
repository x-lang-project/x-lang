use super::*;

pub fn source_declaration_mismatch() -> CompilationFailure {
    InternalDiagnostic::SourceDeclarationMismatch
        .into_failure(source_declaration_metadata_does_not_match_the_parsed_module())
}

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
