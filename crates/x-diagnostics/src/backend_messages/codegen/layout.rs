use std::fmt::{Debug, Display};

use crate::Error;

pub fn allocate_name_returned_void(allocate_name: impl Display) -> String {
    Error::new(format!("{allocate_name} returned void")).render_detail()
}
