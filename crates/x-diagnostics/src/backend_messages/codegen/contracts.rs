use std::fmt::{Debug, Display};

use crate::Error;

pub fn memoised_return_slot_is_missing() -> String {
    Error::new("memoised return slot is missing").render_detail()
}

pub fn memoised_return_type_is_missing() -> String {
    Error::new("memoised return type is missing").render_detail()
}

pub fn memoised_parameter_layout_is_missing() -> String {
    Error::new("memoised parameter layout is missing").render_detail()
}

pub fn memoised_parameter_slot_is_missing() -> String {
    Error::new("memoised parameter slot is missing").render_detail()
}

pub fn memoisation_key_is_not_a_scalar_bit_value() -> String {
    Error::new("memoisation key is not a scalar bit value").render_detail()
}

pub fn invalid_typed_memoise_call_contract_reached_code_generation() -> String {
    Error::new("invalid typed memoise call contract reached code generation").render_detail()
}

pub fn layout_query_function_requires_a_concrete_type(function: impl Display) -> String {
    Error::new(format!(
        "layout query `{function}` requires a concrete type"
    ))
    .render_detail()
}
