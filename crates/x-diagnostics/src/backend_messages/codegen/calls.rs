use std::fmt::{Debug, Display};

use crate::Error;

pub fn function_expects_n_argument_value_got_value(
    function: impl Display,
    count: usize,
    actual_count: usize,
) -> String {
    let plural_suffix = if count == 1 { "" } else { "s" };

    Error::new(format!(
        "`{function}` expects {count} argument{plural_suffix}, got {actual_count}"
    ))
    .render_detail()
}

pub fn value_is_missing_argument_value(
    function_name: impl Display,
    argument_index: impl Display,
) -> String {
    Error::new(format!(
        "`{function_name}` is missing argument {argument_index}"
    ))
    .render_detail()
}

pub fn call_function_returns_void_but_assignment_target_expects_expected_ast_type(
    function: impl Display,
    expected_ast_type: impl Debug,
) -> String {
    Error::new(format!(
        "call `{function}` returns void but assignment target expects {expected_ast_type:?}"
    ))
    .render_detail()
}

pub fn call_function_has_no_matching_typed_signature(function: impl Display) -> String {
    Error::new(format!("call `{function}` has no matching typed signature")).render_detail()
}

pub fn call_function_has_no_resolved_target(function: impl Display) -> String {
    Error::new(format!("call `{function}` has no resolved target")).render_detail()
}

pub fn abort_must_be_a_non_returning_mir_terminator() -> String {
    Error::new("abort must be a non-returning MIR terminator").render_detail()
}

pub fn resolved_intrinsic_value_could_not_emit_function(
    intrinsic: impl Debug,
    function: impl Display,
) -> String {
    Error::new(format!(
        "resolved intrinsic {intrinsic:?} could not emit `{function}`"
    ))
    .render_detail()
}

pub fn cannot_pass_argument_value_to_value_llvm_type_mismatch_value_value(
    argument_index: impl Display,
    function_name: impl Display,
    actual_type: impl Debug,
    expected_type: impl Debug,
) -> String {
    Error::new(format!("cannot pass argument {argument_index} to `{function_name}`: LLVM type mismatch {actual_type:?} -> {expected_type:?}")).render_detail()
}

pub fn cannot_store_llvm_value_value_into_mir_type_value_llvm_type_value(
    operand: impl Debug,
    mir_type: impl Debug,
    llvm_type: impl Debug,
) -> String {
    Error::new(format!(
        "cannot store LLVM value {operand:?} into MIR type {mir_type:?} / LLVM type {llvm_type:?}"
    ))
    .render_detail()
}

pub fn function_expects_arity_argument_s_got_value(
    function: impl Display,
    arity: impl Display,
    actual_count: impl Display,
) -> String {
    Error::new(format!(
        "`{function}` expects {arity} argument(s), got {actual_count}"
    ))
    .render_detail()
}

pub fn symbol_returned_void(symbol: impl Display) -> String {
    Error::new(format!("{symbol} returned void")).render_detail()
}

pub fn bit_intrinsic_llvm_name_did_not_return_a_value(llvm_name: impl Display) -> String {
    Error::new(format!(
        "bit intrinsic `{llvm_name}` did not return a value"
    ))
    .render_detail()
}

pub fn checked_call_remained_a_statement_instead_of_a_mir_terminator() -> String {
    Error::new("checked call remained a statement instead of a MIR terminator").render_detail()
}

pub fn void_call_function_used_where_a_value_was_required(function: impl Display) -> String {
    Error::new(format!(
        "void call `{function}` used where a value was required"
    ))
    .render_detail()
}

pub fn missing_current_llvm_function() -> String {
    Error::new("missing current LLVM function").render_detail()
}

pub fn missing_return_slot_layout_tracking_record() -> String {
    Error::new("missing return-slot layout").render_detail()
}

pub fn throw_terminator_escaped_a_non_throws_function() -> String {
    Error::new("throw terminator escaped a non-throws function").render_detail()
}

pub fn become_function_returned_void_for_non_void_function(function: impl Display) -> String {
    Error::new(format!(
        "`become {function}` returned void for non-void function"
    ))
    .render_detail()
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
