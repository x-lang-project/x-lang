use std::fmt::{Debug, Display};

use crate::Error;

pub fn mir_validation_non_finite_float(function: impl Display, block: impl Display) -> String {
    Error::new(format!(
        "function `{function}` block {block} contains a non-finite float constant"
    ))
    .with_code("M1009")
    .render_detail()
}

pub fn mir_validation_byte_character_output_of_range(
    function: impl Display,
    block: impl Display,
) -> String {
    Error::new(format!(
        "function `{function}` block {block} contains a character outside the byte-character range"
    ))
    .with_code("M1011")
    .render_detail()
}
