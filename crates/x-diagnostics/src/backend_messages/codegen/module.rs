use std::fmt::{Debug, Display};

use crate::Error;

pub fn global_name_has_unsized_array_type_fixed_storage_must_be_lowered_as(
    name: impl Display,
) -> String {
    Error::new(format!(
        "global `{name}` has unsized Array type; fixed storage must be lowered as FixedSizedArray"
    ))
    .render_detail()
}

pub fn missing_execution_engine() -> String {
    Error::new("missing execution engine").render_detail()
}

pub fn jit_entry_name_must_have_a_body_and_a_non_variadic_zero(name: impl Display) -> String {
    Error::new(format!(
        "JIT entry `{name}` must have a body and a non-variadic, zero-argument C ABI"
    ))
    .render_detail()
}

pub fn invalid_jit_module_error(error: impl Display) -> String {
    Error::new(format!("invalid JIT module: {error}")).render_detail()
}

pub fn module_verification_before_optimisation_failed_error(error: impl Display) -> String {
    Error::new(format!(
        "module verification before optimisation failed: {error}"
    ))
    .render_detail()
}

pub fn llvm_optimisation_failed_e(error: impl Display) -> String {
    Error::new(format!("LLVM optimisation failed: {error}")).render_detail()
}

pub fn module_verification_after_optimisation_failed_error(error: impl Display) -> String {
    Error::new(format!(
        "module verification after optimisation failed: {error}"
    ))
    .render_detail()
}

pub fn failed_to_create_target_from_triple_error(error: impl Display) -> String {
    Error::new(format!("failed to create target from triple: {error}")).render_detail()
}

pub fn failed_to_create_llvm_target_machine() -> String {
    Error::new("failed to create LLVM target machine").render_detail()
}

pub fn target_text_is_not_enabled_in_this_x_compiler_build(text: impl Display) -> String {
    Error::new(format!(
        "target `{text}` is not enabled in this X compiler build"
    ))
    .render_detail()
}

pub fn final_target_specific_module_verification_before_object_emission_failed_error(
    error: impl Display,
) -> String {
    Error::new(format!(
        "target module verification before object emission failed: {error}"
    ))
    .render_detail()
}

pub fn llvm_object_emission_to_value_failed_error(
    output_path: impl Display,
    error: impl Display,
) -> String {
    Error::new(format!(
        "LLVM object emission to {output_path} failed: {error}"
    ))
    .render_detail()
}

pub fn generated_textual_llvm_ir_could_not_be_parsed_error(error: impl Display) -> String {
    Error::new(format!(
        "generated textual LLVM IR could not be parsed: {error}"
    ))
    .render_detail()
}

pub fn llvm_symbol_identity_collision(
    symbol: impl Display,
    first_source: impl Display,
    second_source: impl Display,
) -> String {
    Error::new(format!("source declarations `{first_source}` and `{second_source}` collide on LLVM symbol `{symbol}`")).render_detail()
}
