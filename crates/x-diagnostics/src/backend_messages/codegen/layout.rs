use std::fmt::{Debug, Display};

use crate::Error;

pub fn cheri_target_data_layout_is_not_valid_utf() -> String {
    Error::new("CHERI target data layout is not valid UTF-8").render_detail()
}

pub fn cheri_bounds_intrinsic_returned_no_capability() -> String {
    Error::new("CHERI bounds intrinsic returned no capability").render_detail()
}

pub fn cheri_permission_intrinsic_returned_no_capability() -> String {
    Error::new("CHERI permission intrinsic returned no capability").render_detail()
}

pub fn array_helper_element_type(element_type: impl Display) -> String {
    Error::new(format!(
        "array helpers require i32 keys, got `{element_type}`"
    ))
    .render_detail()
}

pub fn allocate_name_returned_void(allocate_name: impl Display) -> String {
    Error::new(format!("{allocate_name} returned void")).render_detail()
}
