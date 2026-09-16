use inkwell::module::Module;

use crate::errors::CodeGenerationError;
use x_diagnostics::backend_messages;

pub(super) enum VerificationStage {
    BeforeOptimization,
    AfterOptimization,
    BeforeObjectEmission,
}

pub(super) fn verify_module(
    module: &Module<'_>,
    stage: VerificationStage,
) -> Result<(), CodeGenerationError> {
    module.verify().map_err(|error| {
        let message = match stage {
            VerificationStage::BeforeOptimization => {
                backend_messages::module_verification_before_optimisation_failed_error(error)
            }
            VerificationStage::AfterOptimization => {
                backend_messages::module_verification_after_optimisation_failed_error(error)
            }
            VerificationStage::BeforeObjectEmission => {
                backend_messages::final_target_specific_module_verification_before_object_emission_failed_error(error)
            }
        };

        CodeGenerationError::LlvmVerificationFailure(message)
    })
}
