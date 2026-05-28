use super::*;

pub fn size_expression_in_function(function: &str) -> String {
    Error::new(format!("size expression in `{function}`")).render_detail()
}

pub fn source_declaration_metadata_does_not_match_the_parsed_module() -> String {
    Error::new("source declaration metadata does not match the parsed module").render_detail()
}

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
