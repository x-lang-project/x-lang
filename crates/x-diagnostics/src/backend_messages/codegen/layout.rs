use std::fmt::{Debug, Display};

use crate::Error;

pub fn fixed_array_pointer_guard_accepted_unexpected(unexpected: impl Debug) -> String {
    Error::new(format!(
        "fixed-array pointer guard accepted `{unexpected:?}`"
    ))
    .render_detail()
}

pub fn cheri_target_data_layout_is_not_valid_utf() -> String {
    Error::new("CHERI target data layout is not valid UTF-8").render_detail()
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
