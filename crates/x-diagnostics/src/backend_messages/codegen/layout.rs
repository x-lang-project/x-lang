use std::fmt::{Debug, Display};

use crate::Error;

pub fn option_has_no_discriminant_preserving_executable_abi_yet() -> String {
    Error::new("Option has no discriminant-preserving executable ABI yet").render_detail()
}

pub fn result_has_no_discriminant_preserving_executable_abi_yet() -> String {
    Error::new("Result has no discriminant-preserving executable ABI yet").render_detail()
}

pub fn mem_layout() -> String {
    Error::new("mem::Layout").render_detail()
}

pub fn type_name_has_no_discriminant_preserving_executable_abi_yet(
    type_name: impl Display,
) -> String {
    Error::new(format!(
        "{type_name} has no discriminant-preserving executable ABI yet"
    ))
    .render_detail()
}

pub fn unknown_field_field_in_structure_name(field: impl Display, name: impl Display) -> String {
    Error::new(format!("unknown field `{field}` in structure `{name}`")).render_detail()
}

pub fn field_name_field_has_no_concrete_layout(name: impl Display, field: impl Display) -> String {
    Error::new(format!("field `{name}::{field}` has no concrete layout")).render_detail()
}

pub fn cannot_determine_offset_of_name_field(name: impl Display, field: impl Display) -> String {
    Error::new(format!("cannot determine offset of `{name}::{field}`")).render_detail()
}

pub fn field_field_name_is_not_present_in_soa_element_element_type(
    field_name: impl Display,
    element_type: impl Debug,
) -> String {
    Error::new(format!(
        "field `{field_name}` is not present in SoA element `{element_type:?}`"
    ))
    .render_detail()
}

pub fn soa_storage_is_missing_field_array_field_index(field_index: impl Display) -> String {
    Error::new(format!("SoA storage is missing field array {field_index}")).render_detail()
}

pub fn named_field_field_name_requires_a_concrete_struct_base_found_base_type(
    field_name: impl Display,
    base_type: impl Debug,
) -> String {
    Error::new(format!(
        "named field `{field_name}` requires a concrete struct base, found {base_type:?}"
    ))
    .render_detail()
}

pub fn field_field_name_not_found_on_struct_canonical_struct_name(
    field_name: impl Display,
    canonical_struct_name: impl Display,
) -> String {
    Error::new(format!(
        "field `{field_name}` not found on struct `{canonical_struct_name}`"
    ))
    .render_detail()
}

pub fn cannot_project_field_field_name_from_non_struct_type_aggregate_ast_type(
    field_name: impl Display,
    aggregate_ast_type: impl Debug,
) -> String {
    Error::new(format!(
        "cannot project field `{field_name}` from non-struct type `{aggregate_ast_type:?}`"
    ))
    .render_detail()
}

pub fn a_layout_soa_array_element_has_no_contiguous_whole_value_address_project() -> String {
    Error::new("a layout(SoA) array element has no contiguous whole-value address; project a field such as `items[i].x`").render_detail()
}

pub fn fixed_array_pointer_guard_accepted_unexpected(unexpected: impl Debug) -> String {
    Error::new(format!(
        "fixed-array pointer guard accepted `{unexpected:?}`"
    ))
    .render_detail()
}

pub fn duplicate_struct_initializer_field_name_field_name(
    name: impl Display,
    field_name: impl Display,
) -> String {
    Error::new(format!(
        "duplicate struct initialiser field `{name}::{field_name}`"
    ))
    .render_detail()
}

pub fn unknown_struct_initializer_field_name_field(
    name: impl Display,
    field: impl Display,
) -> String {
    Error::new(format!(
        "unknown struct initialiser field `{name}::{field}`"
    ))
    .render_detail()
}

pub fn cannot_instantiate_unresolved_struct_name(name: impl Display) -> String {
    Error::new(format!("cannot instantiate unresolved struct `{name}`")).render_detail()
}

pub fn soa_array_initializer_has_value_elements_but_its_type_requires_length(
    element_count: impl Display,
    length: impl Display,
) -> String {
    Error::new(format!(
        "SoA array initialiser has {element_count} elements but its type requires {length}"
    ))
    .render_detail()
}

pub fn soa_element_element_index_is_not_a_struct_value(element_index: impl Display) -> String {
    Error::new(format!("SoA element {element_index} is not a struct value")).render_detail()
}

pub fn cheri_target_data_layout_is_not_valid_utf() -> String {
    Error::new("CHERI target data layout is not valid UTF-8").render_detail()
}

pub fn cannot_create_a_bounded_safe_reference_to_a_zero_sized_object() -> String {
    Error::new("cannot create a bounded safe reference to a zero-sized object").render_detail()
}

pub fn cheri_bounds_intrinsic_returned_no_capability() -> String {
    Error::new("CHERI bounds intrinsic returned no capability").render_detail()
}

pub fn cheri_permission_intrinsic_returned_no_capability() -> String {
    Error::new("CHERI permission intrinsic returned no capability").render_detail()
}

pub fn function_expects_a_slice_receiver(function: impl Display) -> String {
    Error::new(format!("`{function}` expects a slice receiver")).render_detail()
}

pub fn array_helper_element_type(element_type: impl Display) -> String {
    Error::new(format!(
        "array helpers require i32 keys, got `{element_type}`"
    ))
    .render_detail()
}

pub fn slice_descriptor_type_has_no_concrete_size() -> String {
    Error::new("slice descriptor type has no concrete size").render_detail()
}

pub fn allocate_name_returned_void(allocate_name: impl Display) -> String {
    Error::new(format!("{allocate_name} returned void")).render_detail()
}

pub fn slice_length_must_be_an_integer() -> String {
    Error::new("slice length must be an integer").render_detail()
}

pub fn slice_descriptor_data_must_be_a_pointer() -> String {
    Error::new(
        "slice descriptor data must be a pointer; use an explicit cast for integer addresses",
    )
    .render_detail()
}

pub fn soa_field_storage_missing() -> String {
    Error::new("recognised SoA element has no field storage").render_detail()
}

pub fn soa_logical_element_missing() -> String {
    Error::new("recognised SoA element has no storage").render_detail()
}
