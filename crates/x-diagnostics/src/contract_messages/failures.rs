use super::*;

pub fn memoized_function_impure(name: &str) -> CompilationFailure {
    OperationalDiagnostic::MemoizedFunctionImpure
        .into_failure(memoise_function_name_must_also_be_pure(name))
}

pub fn memoized_function_throws(name: &str) -> CompilationFailure {
    OperationalDiagnostic::MemoizedFunctionThrows
        .into_failure(memoise_function_name_cannot_throw(name))
}

pub fn memoized_multi_function(name: &str) -> CompilationFailure {
    OperationalDiagnostic::MemoizedMultiFunction
        .into_failure(memoise_and_multi_cannot_be_combined_on_name(name))
}

pub fn memoized_function_non_scalar(name: &str) -> CompilationFailure {
    OperationalDiagnostic::MemoizedFunctionNonScalar.into_failure(
        memoise_function_name_requires_scalar_numeric_boolean_or_char_parameters_and_return(name),
    )
}

pub fn multi_function_dispatch_missing(name: &str) -> CompilationFailure {
    OperationalDiagnostic::MultiFunctionDispatchMissing
        .into_failure(multi_function_name_requires_a_non_static_dispatch_parameter(name))
}

pub fn parallel_body_unsupported() -> CompilationFailure {
    OperationalDiagnostic::ParallelBodyUnsupported
        .into_failure(x_contract_parallel_parallel_range_body_must_be_exactly_one_direct_worker())
}

pub fn parallel_worker_input_invalid() -> CompilationFailure {
    OperationalDiagnostic::ParallelWorkerInputInvalid
        .into_failure(parallel_worker_must_receive_the_loop_variable_first())
}

pub fn parallel_worker_unknown(name: &str) -> CompilationFailure {
    OperationalDiagnostic::ParallelWorkerUnknown
        .into_failure(parallel_worker_name_is_not_a_known_function(name))
}

pub fn parallel_worker_signature_invalid(name: &str) -> CompilationFailure {
    OperationalDiagnostic::ParallelWorkerSignatureInvalid
        .into_failure(parallel_worker_name_must_be_a_pure_non_throwing_function_with_first(name))
}

pub fn parallel_capture_unsupported() -> CompilationFailure {
    OperationalDiagnostic::ParallelCaptureUnsupported.into_failure(
        x_contract_parallel_captures_must_be_invariant_scalar_values_or_transitively_read(),
    )
}

pub fn parallel_foreach_unsupported() -> CompilationFailure {
    OperationalDiagnostic::ParallelForeachUnsupported
        .into_failure(parallel_foreach_is_not_executable_use_a_range_worker())
}

pub fn become_not_final() -> CompilationFailure {
    OperationalDiagnostic::BecomeNotFinal
        .into_failure(become_must_be_the_final_statement_in_its_block())
}

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
