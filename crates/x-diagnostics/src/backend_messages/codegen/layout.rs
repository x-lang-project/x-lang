use std::fmt::{Debug, Display};

use crate::Error;

pub fn array_helper_element_type(element_type: impl Display) -> String {
    Error::new(format!(
        "array helpers require i32 keys, got `{element_type}`"
    ))
    .render_detail()
}

pub fn allocate_name_returned_void(allocate_name: impl Display) -> String {
    Error::new(format!("{allocate_name} returned void")).render_detail()
}
