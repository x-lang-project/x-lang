use std::fmt::{Debug, Display};

use crate::Error;

pub fn c_emission_parallel_statement() -> String {
    Error::new("parallel statement").render_detail()
}

pub fn c_emission_struct_runtime_invariant() -> String {
    Error::new("struct runtime invariant").render_detail()
}
