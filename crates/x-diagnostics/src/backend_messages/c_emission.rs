use std::fmt::{Debug, Display};

use crate::Error;

pub fn c_emission_nonnumeric_tuple_field() -> String {
    Error::new("nonnumeric tuple field").render_detail()
}

pub fn c_emission_nonscalar_memoise_key() -> String {
    Error::new("nonscalar memoise key").render_detail()
}

pub fn c_emission_nonscalar_or_throwing_memoise_result() -> String {
    Error::new("nonscalar or throwing memoise result").render_detail()
}

pub fn c_emission_parallel_statement() -> String {
    Error::new("parallel statement").render_detail()
}

pub fn c_emission_struct_runtime_invariant() -> String {
    Error::new("struct runtime invariant").render_detail()
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

pub fn c_emission_unbounded_memoise_capacity() -> String {
    Error::new("unbounded memoise capacity").render_detail()
}

pub fn c_emission_unknown_nonmemoise_call_contract() -> String {
    Error::new("unknown nonmemoise call contract").render_detail()
}

pub fn c_emission_unsupported_memoise_contract() -> String {
    Error::new("unsupported memoise contract").render_detail()
}
