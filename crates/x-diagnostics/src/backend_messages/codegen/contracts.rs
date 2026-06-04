use std::fmt::{Debug, Display};

use crate::Error;

pub fn layout_query_function_requires_a_concrete_type(function: impl Display) -> String {
    Error::new(format!(
        "layout query `{function}` requires a concrete type"
    ))
    .render_detail()
}
