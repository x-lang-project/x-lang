use super::*;

pub fn failed_to_create_object_output_directory_value_error(
    output_directory: impl Display,
    error: impl Display,
) -> String {
    Error::new(format!(
        "failed to create object output directory {output_directory}: {error}"
    ))
    .render_detail()
}

pub fn failed_to_save_ir_to_value_error(ir_path: impl Display, error: impl Display) -> String {
    Error::new(format!("Failed to save IR to {ir_path}: {error}")).render_detail()
}

pub fn failed_to_save_value_error(ir_path: impl Display, error: impl Display) -> String {
    Error::new(format!("failed to save {ir_path}: {error}")).render_detail()
}
