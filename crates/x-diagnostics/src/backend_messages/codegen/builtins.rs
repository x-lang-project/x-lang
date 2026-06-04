use std::fmt::{Debug, Display};

use crate::Error;

pub fn print_does_not_support_value_of_type_type(value_type: impl Debug) -> String {
    Error::new(format!(
        "print does not support value of type {value_type:?}"
    ))
    .render_detail()
}

pub fn pointer_assert_aligned_has_no_declared_source_level_contract_or_backend_implementation()
-> String {
    Error::new("ptr::assert_aligned has no executable backend implementation").render_detail()
}

pub fn function_requires_exactly_one_compile_time_field_token(function: impl Display) -> String {
    Error::new(format!(
        "`{function}` requires exactly one compile-time field token"
    ))
    .render_detail()
}

pub fn leaf_expects_a_vec_receiver(leaf: impl Display) -> String {
    Error::new(format!("{leaf} expects a Vec receiver")).render_detail()
}

pub fn as_str_expects_a_receiver() -> String {
    Error::new("as_str expects a receiver").render_detail()
}

pub fn str_as_bytes_expects_a_receiver() -> String {
    Error::new("str::as_bytes expects a receiver").render_detail()
}

pub fn unwrap_cannot_execute_until_option_result_preserve_their_discriminant() -> String {
    Error::new("unwrap cannot execute until Option/Result preserve their discriminant")
        .render_detail()
}

pub fn leaf_cannot_execute_until_sum_types_preserve_their_discriminant(
    leaf: impl Display,
) -> String {
    Error::new(format!(
        "{leaf} cannot execute until sum types preserve their discriminant"
    ))
    .render_detail()
}
