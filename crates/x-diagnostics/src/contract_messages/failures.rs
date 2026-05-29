use super::*;

pub fn object_output_directory(
    output_directory: impl Display,
    error: impl Display,
) -> CompilationFailure {
    OperationalDiagnostic::ObjectOutputDirectory.into_failure(
        failed_to_create_object_output_directory_value_error(output_directory, error),
    )
}

pub fn ir_save(ir_path: impl Display, error: impl Display) -> CompilationFailure {
    OperationalDiagnostic::IrSave.into_failure(failed_to_save_ir_to_value_error(ir_path, error))
}

pub fn ir_save_failed_to_save_value_error(
    ir_path: impl Display,
    error: impl Display,
) -> CompilationFailure {
    OperationalDiagnostic::IrSave.into_failure(failed_to_save_value_error(ir_path, error))
}

pub fn resolved_module_missing(mod_identifier: &str) -> CompilationFailure {
    InternalDiagnostic::ResolvedModuleMissing.into_failure(
        resolved_module_order_references_missing_module_mod_identifier(mod_identifier),
    )
}

pub fn source_declaration_mismatch() -> CompilationFailure {
    InternalDiagnostic::SourceDeclarationMismatch
        .into_failure(source_declaration_metadata_does_not_match_the_parsed_module())
}

pub fn source_capacity_exceeded() -> CompilationFailure {
    InternalDiagnostic::SourceCapacityExceeded.into_failure(too_many_source_files())
}

pub fn compiling_module_note(module: &str) -> String {
    format!("while compiling module `{module}`")
}

pub fn module_object_emission_note(module: &str) -> String {
    format!("module `{module}` during LLVM object emission")
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
