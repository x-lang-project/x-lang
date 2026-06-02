use std::fmt::Debug;

use crate::Error;

pub fn operator_operator_escaped_its_validated_backend_class(operator: impl Debug) -> String {
    Error::new(format!(
        "operator `{operator:?}` escaped its validated backend class"
    ))
    .render_detail()
}
