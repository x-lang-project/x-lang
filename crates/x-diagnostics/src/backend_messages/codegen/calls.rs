use std::fmt::{Debug, Display};

use crate::Error;

pub fn abort_must_be_a_non_returning_mir_terminator() -> String {
    Error::new("abort must be a non-returning MIR terminator").render_detail()
}

pub fn missing_return_slot_layout_tracking_record() -> String {
    Error::new("missing return-slot layout").render_detail()
}

pub fn throw_terminator_escaped_a_non_throws_function() -> String {
    Error::new("throw terminator escaped a non-throws function").render_detail()
}

pub fn missing_resume_block_basic_block_value(block: impl Display) -> String {
    Error::new(format!("Missing resume block bb_{block}")).render_detail()
}

pub fn checked_call_function_escaped_a_non_throws_function(function: impl Display) -> String {
    Error::new(format!(
        "checked call `{function}?` escaped a non-throws function"
    ))
    .render_detail()
}

pub fn throwing_call_function_returned_void(function: impl Display) -> String {
    Error::new(format!("throwing call `{function}` returned void")).render_detail()
}

pub fn missing_throws_continuation_basic_block_value(block: impl Display) -> String {
    Error::new(format!("missing throws continuation bb_{block}")).render_detail()
}

pub fn missing_catch_continuation_basic_block_value(block: impl Display) -> String {
    Error::new(format!("missing catch continuation bb_{block}")).render_detail()
}

pub fn throws_function_has_no_aggregate_return_type() -> String {
    Error::new("throws function has no aggregate return type").render_detail()
}

pub fn abort_trap_calls_must_lower_as_non_returning_mir_terminators() -> String {
    Error::new("abort/trap calls must lower as non-returning MIR terminators").render_detail()
}
