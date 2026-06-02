use std::fmt::Debug;

use crate::Error;

pub fn binary_operator_operator_has_non_convertible_operands_left_left_type_lowered_to(
    operator: impl Debug,
    left_type: impl Debug,
    left_value: impl Debug,
    right_value: impl Debug,
) -> String {
    Error::new(format!("operator `{operator:?}` has incompatible operands: left {left_type:?} lowered to {left_value:?}, right lowered to {right_value:?}")).render_detail()
}

pub fn operator_operator_escaped_its_validated_backend_class(operator: impl Debug) -> String {
    Error::new(format!(
        "operator `{operator:?}` escaped its validated backend class"
    ))
    .render_detail()
}
