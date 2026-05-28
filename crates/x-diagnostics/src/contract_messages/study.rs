use super::*;

pub fn too_many_source_files() -> String {
    Error::new("too many source files").render_detail()
}

pub fn resolved_module_order_references_missing_module_mod_identifier(
    mod_identifier: &str,
) -> String {
    Error::new(format!(
        "resolved module order references missing module `{mod_identifier}`"
    ))
    .render_detail()
}
