use super::*;

pub(super) fn angle_contents(text: &str) -> Option<&str> {
    let start = text.find('<')?;
    let end = text.rfind('>')?;

    start
        .checked_add(1)
        .and_then(|contents_start| text.get(contents_start..end))
}

pub(super) fn type_from_trait_argument(text: &str) -> Type {
    x_ast::type_from_name(text)
}
