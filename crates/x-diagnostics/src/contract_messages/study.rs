use super::*;

pub fn parameter_of_function_name(name: &str) -> String {
    Error::new(format!("parameter of function `{name}`")).render_detail()
}

pub fn return_type_of_function_name(name: &str) -> String {
    Error::new(format!("return type of function `{name}`")).render_detail()
}

pub fn cheri_external_function(name: &str) -> String {
    Error::new(format!(
        "CHERI target does not support external function `{name}`"
    ))
    .render_detail()
}

pub fn cheri_throwing_function(name: &str) -> String {
    Error::new(format!(
        "CHERI target does not support throwing function `{name}`"
    ))
    .render_detail()
}

pub fn local_local_name_in_function_name(local_name: &str, name: &str) -> String {
    Error::new(format!("local `{local_name}` in function `{name}`")).render_detail()
}

pub fn cheri_structure_layout(struct_name: &str, layout: impl Debug) -> String {
    Error::new(format!(
        "CHERI target requires AoS layout for structure `{struct_name}`, found {layout:?}"
    ))
    .render_detail()
}

pub fn cheri_generic_structure(struct_name: &str) -> String {
    Error::new(format!(
        "CHERI target requires a concrete structure for `{struct_name}`"
    ))
    .render_detail()
}

pub fn field_field_of_structure_value(field: &str, struct_name: &str) -> String {
    Error::new(format!("field `{field}` of structure `{struct_name}`")).render_detail()
}

pub fn cheri_trait_call(trait_name: &str) -> String {
    Error::new(format!(
        "CHERI target does not support trait calls through `{trait_name}`"
    ))
    .render_detail()
}

pub fn cheri_trait_implementation(trait_name: &str) -> String {
    Error::new(format!(
        "CHERI target does not support trait implementation `{trait_name}`"
    ))
    .render_detail()
}

pub fn cheri_static(name: &str) -> String {
    Error::new(format!("CHERI target does not support static `{name}`")).render_detail()
}

pub fn safe_reference_temporary_in_function_function(function: &str) -> String {
    Error::new(format!("safe-reference temporary in function `{function}`")).render_detail()
}

pub fn compiler_temporary_in_function_function(function: &str) -> String {
    Error::new(format!("compiler temporary in function `{function}`")).render_detail()
}

pub fn dynamic_slice_in_context(context: &str) -> String {
    Error::new(format!("dynamic slice in {context}")).render_detail()
}

pub fn managed_string_in_context(context: &str) -> String {
    Error::new(format!("managed string in {context}")).render_detail()
}

pub fn raw_pointer_in_context(context: &str) -> String {
    Error::new(format!("raw pointer in {context}")).render_detail()
}

pub fn tuple_in_context(context: &str) -> String {
    Error::new(format!("tuple in {context}")).render_detail()
}

pub fn simd_value_in_context(context: &str) -> String {
    Error::new(format!("SIMD value in {context}")).render_detail()
}

pub fn function_value_or_closure_in_context(context: &str) -> String {
    Error::new(format!("function value or closure in {context}")).render_detail()
}

pub fn unresolved_generic_type_in_context(context: &str) -> String {
    Error::new(format!("unresolved generic type in {context}")).render_detail()
}

pub fn unresolved_type_in_context(context: &str) -> String {
    Error::new(format!("unresolved type in {context}")).render_detail()
}

pub fn cheri_parallel_loop(function: &str) -> String {
    Error::new(format!(
        "CHERI target does not support parallel loops in `{function}`"
    ))
    .render_detail()
}

pub fn pointer_arithmetic_in_function_function(function: &str) -> String {
    Error::new(format!("pointer arithmetic in function `{function}`")).render_detail()
}

pub fn cast_to_pointer_or_safe_reference_in_function_function(function: &str) -> String {
    Error::new(format!(
        "cast to pointer or safe reference in function `{function}`"
    ))
    .render_detail()
}

pub fn cast_in_function_function(function: &str) -> String {
    Error::new(format!("cast in function `{function}`")).render_detail()
}

pub fn tuple_construction_in_function_function(function: &str) -> String {
    Error::new(format!("tuple construction in function `{function}`")).render_detail()
}

pub fn size_expression_in_function(function: &str) -> String {
    Error::new(format!("size expression in `{function}`")).render_detail()
}

pub fn exception_operation_in_function_function(function: &str) -> String {
    Error::new(format!("exception operation in function `{function}`")).render_detail()
}

pub fn exception_control_flow_in_function_function(function: &str) -> String {
    Error::new(format!("exception control flow in function `{function}`")).render_detail()
}

pub fn string_value_in_function_function(function: &str) -> String {
    Error::new(format!("string value in function `{function}`")).render_detail()
}

pub fn type_literal_in_function_function(function: &str) -> String {
    Error::new(format!("type literal in function `{function}`")).render_detail()
}

pub fn cheri_global_place(name: &str, function: &str) -> String {
    Error::new(format!(
        "CHERI target does not support global place `{name}` in `{function}`"
    ))
    .render_detail()
}

pub fn projected_place_in_function_function(function: &str) -> String {
    Error::new(format!("projected place in function `{function}`")).render_detail()
}

pub fn source_declaration_metadata_does_not_match_the_parsed_module() -> String {
    Error::new("source declaration metadata does not match the parsed module").render_detail()
}

pub fn too_many_source_files() -> String {
    Error::new("too many source files").render_detail()
}

pub fn resolved_module_order_references_missing_module_mod_identifier(
    mod_identifier: &str,
) -> String {
    Error::new(format!(
        "resolved module order references missing module `{mod_identifier}`"
    ))
    .render_detail()
}
