use std::fmt::{Debug, Display};

use crate::Error;

pub fn static_name(name: impl Display) -> String {
    Error::new(format!("static `{name}`")).render_detail()
}

pub fn struct_value_field_field(struct_name: impl Display, field: impl Display) -> String {
    Error::new(format!("struct `{struct_name}` field `{field}`")).render_detail()
}

pub fn mir_validation_missing_entry_block(function: impl Display) -> String {
    Error::new(format!("function `{function}` has no entry block 0"))
        .with_code("M1001")
        .render_detail()
}

pub fn mir_validation_local_identity_mismatch(
    function: impl Display,
    key: impl Display,
    declared: impl Display,
) -> String {
    Error::new(format!(
        "function `{function}` local map key {key} disagrees with declaration {declared}"
    ))
    .with_code("M1004")
    .render_detail()
}

pub fn mir_validation_missing_terminator(function: impl Display, block: impl Display) -> String {
    Error::new(format!(
        "function `{function}` block {block} has no terminator"
    ))
    .with_code("M1005")
    .render_detail()
}

pub fn mir_validation_missing_target_block(
    function: impl Display,
    block: impl Display,
    target: impl Display,
) -> String {
    Error::new(format!(
        "function `{function}` block {block} targets missing block {target}"
    ))
    .with_code("M1006")
    .render_detail()
}

pub fn mir_validation_missing_local(
    function: impl Display,
    block: impl Display,
    local: impl Display,
) -> String {
    Error::new(format!(
        "function `{function}` block {block} references missing local {local}"
    ))
    .with_code("M1007")
    .render_detail()
}

pub fn mir_validation_unresolved_type(context: impl Display, value_type: impl Debug) -> String {
    Error::new(format!(
        "`{context}` contains unresolved type `{value_type:?}`"
    ))
    .with_code("M1008")
    .render_detail()
}

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
