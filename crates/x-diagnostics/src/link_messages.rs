use std::fmt::Display;

use crate::CompilationFailure;
use crate::Error;
use crate::LinkDiagnostic;

pub fn linking_failed_with_status_status(status: impl Display) -> String {
    Error::new(format!("Linking failed with status {status}")).render_detail()
}

pub fn configured_by_selected_value_but_it_could_not_be_executed(
    configured_by: impl Display,
    clang_path: impl Display,
) -> String {
    Error::new(format!(
        "{configured_by} selected {clang_path}, but it could not be executed"
    ))
    .render_detail()
}
