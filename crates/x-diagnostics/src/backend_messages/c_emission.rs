use std::fmt::{Debug, Display};

use crate::Error;

pub fn c_emission_c_source_formatting_failed() -> String {
    Error::new("C source formatting failed").render_detail()
}

pub fn c_emission_float_bitwise() -> String {
    Error::new("float bitwise").render_detail()
}

pub fn c_emission_float_operator() -> String {
    Error::new("float operator").render_detail()
}

pub fn c_emission_helper_operation_is_missing() -> String {
    Error::new("helper operation is missing").render_detail()
}

pub fn c_emission_helper_signedness_is_missing() -> String {
    Error::new("helper signedness is missing").render_detail()
}

pub fn c_emission_helper_width_is_invalid() -> String {
    Error::new("helper width is invalid").render_detail()
}

pub fn c_emission_helper_width_is_missing() -> String {
    Error::new("helper width is missing").render_detail()
}

pub fn c_emission_integer_literal_width_is_missing() -> String {
    Error::new("integer literal width is missing").render_detail()
}

pub fn c_emission_missing_reachable_block() -> String {
    Error::new("missing reachable block").render_detail()
}

pub fn c_emission_missing_reachable_terminator() -> String {
    Error::new("missing reachable terminator").render_detail()
}

pub fn c_emission_missing_terminator() -> String {
    Error::new("missing terminator").render_detail()
}

pub fn c_emission_missing_throwing_callee() -> String {
    Error::new("missing throwing callee").render_detail()
}

pub fn c_emission_non_source_throwing_call() -> String {
    Error::new("non-source throwing call").render_detail()
}

pub fn c_emission_nonfinite_literal() -> String {
    Error::new("nonfinite literal").render_detail()
}

pub fn c_emission_noninteger_switch() -> String {
    Error::new("noninteger switch").render_detail()
}

pub fn c_emission_nonnumeric_tuple_field() -> String {
    Error::new("nonnumeric tuple field").render_detail()
}

pub fn c_emission_nonscalar_memoise_key() -> String {
    Error::new("nonscalar memoise key").render_detail()
}

pub fn c_emission_nonscalar_or_throwing_memoise_result() -> String {
    Error::new("nonscalar or throwing memoise result").render_detail()
}

pub fn c_emission_numeric_value_width_is_missing() -> String {
    Error::new("numeric value width is missing").render_detail()
}

pub fn c_emission_parallel_statement() -> String {
    Error::new("parallel statement").render_detail()
}

pub fn c_emission_struct_runtime_invariant() -> String {
    Error::new("struct runtime invariant").render_detail()
}

pub fn c_emission_throw_outside_throws_body() -> String {
    Error::new("throw outside throws body").render_detail()
}

pub fn c_emission_throws_callee_contract_arity() -> String {
    Error::new("throws callee contract/arity").render_detail()
}

pub fn c_emission_try_outside_throws_body() -> String {
    Error::new("try outside throws body").render_detail()
}

pub fn c_emission_tuple_arity() -> String {
    Error::new("tuple arity").render_detail()
}

pub fn c_emission_tuple_field_bounds() -> String {
    Error::new("tuple field bounds").render_detail()
}

pub fn c_emission_tuple_target_layout() -> String {
    Error::new("tuple target layout").render_detail()
}

pub fn c_emission_unary_effect() -> String {
    Error::new("unary effect").render_detail()
}

pub fn c_emission_unbounded_memoise_capacity() -> String {
    Error::new("unbounded memoise capacity").render_detail()
}

pub fn c_emission_unchecked_throwing_call() -> String {
    Error::new("unchecked throwing call").render_detail()
}

pub fn c_emission_unknown_nonmemoise_call_contract() -> String {
    Error::new("unknown nonmemoise call contract").render_detail()
}

pub fn c_emission_unreachable_terminator() -> String {
    Error::new("unreachable terminator").render_detail()
}

pub fn c_emission_unsupported_memoise_contract() -> String {
    Error::new("unsupported memoise contract").render_detail()
}

pub fn c_emission_void_catch_destination() -> String {
    Error::new("void catch destination").render_detail()
}

pub fn c_emission_write_to_string_element() -> String {
    Error::new("write to string element").render_detail()
}

pub fn c_emission_unsupported_conversion(value: impl Debug, target: impl Debug) -> String {
    Error::new(format!("unsupported conversion {value:?}->{target:?}")).render_detail()
}
