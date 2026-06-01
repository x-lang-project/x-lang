use std::fmt::{Debug, Display};

use crate::Error;

pub fn parallel_range_body_must_be_one_direct_worker_call() -> String {
    Error::new("parallel range body must be one direct worker call").render_detail()
}

pub fn parallel_worker_must_receive_the_loop_variable_first() -> String {
    Error::new("parallel worker must receive the loop variable first").render_detail()
}

pub fn parallel_range_requires_i32_bounds() -> String {
    Error::new("parallel runtime requires range bounds of signed i32 type").render_detail()
}
