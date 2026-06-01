use std::fmt::{Debug, Display};

use crate::Error;

pub fn deref_source_is_not_a_place_operand_other(other: impl Debug) -> String {
    Error::new(format!("deref source is not a place operand: {other:?}")).render_detail()
}

pub fn array_base_is_not_a_place_operand_other(other: impl Debug) -> String {
    Error::new(format!("array base is not a place operand: {other:?}")).render_detail()
}

pub fn field_object_is_not_a_place_operand_other(other: impl Debug) -> String {
    Error::new(format!("field object is not a place operand: {other:?}")).render_detail()
}

pub fn postfix_did_not_lower_to_a_direct_function_call() -> String {
    Error::new("postfix `?` did not lower to a direct function call").render_detail()
}

pub fn postfix_requires_a_direct_throwing_call() -> String {
    Error::new("postfix `?` requires a direct throwing call").render_detail()
}

pub fn if_expression_lowering_requires_expression_valued_branches() -> String {
    Error::new("if expression lowering requires expression-valued branches").render_detail()
}

pub fn expression_lowering_is_not_implemented_for_this_shape_other(other: impl Debug) -> String {
    Error::new(format!(
        "expression lowering is not implemented for this shape: {other:?}"
    ))
    .render_detail()
}

pub fn irrefutable_matches_require_statement_aware_lowering() -> String {
    Error::new("irrefutable matches require statement-aware lowering").render_detail()
}

pub fn match_expression_lowering_is_not_implemented_for_this_shape_arms(
    arms: impl Debug,
) -> String {
    Error::new(format!(
        "match expression lowering is not implemented for this shape: {arms:?}"
    ))
    .render_detail()
}

pub fn payload_binding_arity_exceeds_tuple_fields() -> String {
    Error::new("payload binding arity exceeds tuple fields").render_detail()
}

pub fn catch_is_missing_its_throwing_call() -> String {
    Error::new("catch is missing its throwing call").render_detail()
}

pub fn catch_is_missing_its_fallback_value() -> String {
    Error::new("catch is missing its fallback value").render_detail()
}

pub fn catch_requires_a_direct_throwing_call() -> String {
    Error::new("catch requires a direct throwing call").render_detail()
}

pub fn catch_fallback_integer_overflows() -> String {
    Error::new("catch fallback integer overflows").render_detail()
}

pub fn catch_fallback_must_already_be_evaluated() -> String {
    Error::new("catch fallback must already be evaluated").render_detail()
}

pub fn some_constructor_lost_its_validated_argument() -> String {
    Error::new("Some constructor lost its validated argument").render_detail()
}

pub fn expression_is_not_addressable_other(other: impl Debug) -> String {
    Error::new(format!("expression is not addressable: {other:?}")).render_detail()
}

pub fn deref_source_is_not_a_resolvable_place_operand_other(other: impl Debug) -> String {
    Error::new(format!(
        "deref source is not a resolvable place operand: {other:?}"
    ))
    .render_detail()
}

pub fn context_is_not_a_resolvable_place_operand_other(
    context: impl Display,
    other: impl Debug,
) -> String {
    Error::new(format!(
        "{context} is not a resolvable place operand: {other:?}"
    ))
    .render_detail()
}

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

pub fn tuple_pattern_requires_a_tuple() -> String {
    Error::new("tuple pattern requires a tuple").render_detail()
}

pub fn tuple_pattern_arity_mismatch() -> String {
    Error::new("tuple pattern arity mismatch").render_detail()
}

pub fn expected_an_irrefutable_pattern() -> String {
    Error::new("expected an irrefutable pattern").render_detail()
}

pub fn lower_match_statement_called_for_a_non_match_expression() -> String {
    Error::new("lower_match_statement called for a non-match expression").render_detail()
}

pub fn match_statement_has_no_arms() -> String {
    Error::new("match statement has no arms").render_detail()
}

pub fn assignment_destination_is_not_addressable_other(other: impl Debug) -> String {
    Error::new(format!(
        "assignment destination is not addressable: {other:?}"
    ))
    .render_detail()
}

pub fn internal_call_index_did_not_identify_an_assignment() -> String {
    Error::new("internal call index did not identify an assignment").render_detail()
}

pub fn internal_call_index_did_not_identify_a_try_or_catch_call() -> String {
    Error::new("internal call index did not identify a try or catch call").render_detail()
}

pub fn serial_range_requires_integer_bounds() -> String {
    Error::new("range loops require integer bounds").render_detail()
}

pub fn foreach_is_not_executable_use_a_range_loop() -> String {
    Error::new("foreach is not executable; use a range loop").render_detail()
}

pub fn parallel_range_body_must_be_one_direct_worker_call() -> String {
    Error::new("parallel range body must be one direct worker call").render_detail()
}

pub fn parallel_worker_must_receive_the_loop_variable_first() -> String {
    Error::new("parallel worker must receive the loop variable first").render_detail()
}

pub fn break_is_only_executable_inside_a_loop() -> String {
    Error::new("`break` is only executable inside a loop").render_detail()
}

pub fn continue_is_only_executable_inside_a_loop() -> String {
    Error::new("`continue` is only executable inside a loop").render_detail()
}

pub fn become_requires_a_direct_function_call() -> String {
    Error::new("`become` requires a direct function call").render_detail()
}

pub fn expected_if_expression() -> String {
    Error::new("expected if expression").render_detail()
}

pub fn value_producing_if_requires_an_else_branch() -> String {
    Error::new("value-producing if requires an else branch").render_detail()
}

pub fn continuing_value_branch_requires_a_tail_expression() -> String {
    Error::new("continuing value branch requires a tail expression").render_detail()
}

pub fn parallel_range_requires_i32_bounds() -> String {
    Error::new("parallel runtime requires range bounds of signed i32 type").render_detail()
}
