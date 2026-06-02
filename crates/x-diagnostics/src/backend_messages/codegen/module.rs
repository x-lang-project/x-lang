use std::fmt::{Debug, Display};

use crate::Error;

pub fn invalid_jit_module_error(error: impl Display) -> String {
    Error::new(format!("invalid JIT module: {error}")).render_detail()
}

pub fn module_verification_before_optimisation_failed_error(error: impl Display) -> String {
    Error::new(format!(
        "module verification before optimisation failed: {error}"
    ))
    .render_detail()
}

pub fn module_verification_after_optimisation_failed_error(error: impl Display) -> String {
    Error::new(format!(
        "module verification after optimisation failed: {error}"
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
