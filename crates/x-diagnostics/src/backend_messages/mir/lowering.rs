use std::fmt::{Debug, Display};

use crate::Error;

pub fn control_flow_evaluation_is_not_implemented_inside_shape(shape: impl Debug) -> String {
    Error::new(format!(
        "control-flow evaluation is not implemented inside {shape:?}"
    ))
    .render_detail()
}

pub fn array_repetition_length_must_remain_a_compile_time_integer_through_mir_lowering() -> String {
    Error::new("array repetition length must remain a compile-time integer through MIR lowering")
        .render_detail()
}

pub fn array_repetition_length_must_be_non_negative_and_fit_usize() -> String {
    Error::new("array repetition length must be non-negative and fit usize").render_detail()
}

pub fn expected_an_irrefutable_pattern() -> String {
    Error::new("expected an irrefutable pattern").render_detail()
}

pub fn serial_range_requires_integer_bounds() -> String {
    Error::new("range loops require integer bounds").render_detail()
}

pub fn parallel_range_body_must_be_one_direct_worker_call() -> String {
    Error::new("parallel range body must be one direct worker call").render_detail()
}

pub fn parallel_worker_must_receive_the_loop_variable_first() -> String {
    Error::new("parallel worker must receive the loop variable first").render_detail()
}

pub fn parallel_range_requires_i32_bounds() -> String {
    Error::new("parallel runtime requires range bounds of signed i32 type").render_detail()
}
