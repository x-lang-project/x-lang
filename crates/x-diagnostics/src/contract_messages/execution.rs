use super::*;

pub fn pure_function_name_cannot_throw(name: &str) -> String {
    Error::new(format!("`pure` function `{name}` cannot throw")).render_detail()
}

pub fn memoise_function_name_must_also_be_pure(name: &str) -> String {
    Error::new(format!("`memoise` function `{name}` must also be pure")).render_detail()
}

pub fn memoise_function_name_cannot_throw(name: &str) -> String {
    Error::new(format!("`memoise` function `{name}` cannot throw")).render_detail()
}

pub fn memoise_and_multi_cannot_be_combined_on_name(name: &str) -> String {
    Error::new(format!(
        "`memoise` and `multi` cannot be combined on `{name}`"
    ))
    .render_detail()
}

pub fn memoise_function_name_requires_scalar_numeric_boolean_or_char_parameters_and_return(
    name: &str,
) -> String {
    Error::new(format!("`memoise` function `{name}` requires scalar numeric, bool, or char parameters and return value")).render_detail()
}

pub fn multi_function_name_requires_a_non_static_dispatch_parameter(name: &str) -> String {
    Error::new(format!(
        "`multi` function `{name}` requires a non-static dispatch parameter"
    ))
    .render_detail()
}

pub fn x_contract_invariant_struct_invariant_for_value_cannot_contain_calls_assignments_dereferences(
    struct_name: &str,
) -> String {
    Error::new(format!("x-contract-invariant: struct invariant for `{struct_name}` cannot contain calls, assignments, dereferences, or other unsupported expressions")).render_detail()
}

pub fn x_contract_parallel_parallel_range_body_must_be_exactly_one_direct_worker() -> String {
    Error::new("x-contract-parallel: `parallel` range body must be exactly one direct worker call")
        .render_detail()
}

pub fn parallel_worker_must_receive_the_loop_variable_first() -> String {
    Error::new("`parallel` worker must receive the loop variable first").render_detail()
}

pub fn parallel_worker_name_is_not_a_known_function(name: &str) -> String {
    Error::new(format!(
        "`parallel` worker `{name}` is not a known function"
    ))
    .render_detail()
}

pub fn parallel_worker_name_must_be_a_pure_non_throwing_function_with_first(name: &str) -> String {
    Error::new(format!("`parallel` worker `{name}` must be a pure non-throwing function with first parameter i32 and result i32")).render_detail()
}

pub fn x_contract_parallel_captures_must_be_invariant_scalar_values_or_transitively_read() -> String
{
    Error::new("x-contract-parallel: captures must be invariant scalar values or transitively read-only borrowed storage; raw pointers and mutable/unproved aliases are rejected").render_detail()
}

pub fn parallel_foreach_is_not_executable_use_a_range_worker() -> String {
    Error::new("`parallel` foreach is not executable; use a range worker").render_detail()
}

pub fn become_must_be_the_final_statement_in_its_block() -> String {
    Error::new("`become` must be the final statement in its block").render_detail()
}

pub fn use_x_cheri_build_the_ordinary_llvm_object_path_cannot_certify_the() -> String {
    Error::new("use `x cheri-build`; the ordinary LLVM object path cannot certify the CHERI pure-capability ABI").render_detail()
}

pub fn failed_to_create_object_output_directory_value_error(
    output_directory: impl Display,
    error: impl Display,
) -> String {
    Error::new(format!(
        "failed to create object output directory {output_directory}: {error}"
    ))
    .render_detail()
}

pub fn failed_to_save_ir_to_value_error(ir_path: impl Display, error: impl Display) -> String {
    Error::new(format!("Failed to save IR to {ir_path}: {error}")).render_detail()
}

pub fn failed_to_save_value_error(ir_path: impl Display, error: impl Display) -> String {
    Error::new(format!("failed to save {ir_path}: {error}")).render_detail()
}
