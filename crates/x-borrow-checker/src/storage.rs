use std::collections::HashMap;

use super::ReferenceValue;
use super::Region;
use x_ast::Type;

pub(super) fn field_paths_overlap(first: &str, second: &str) -> bool {
    let descendant_of = |path: &str, parent: &str| {
        path.strip_prefix(parent)
            .is_some_and(|suffix| suffix.starts_with('.'))
    };

    first == second || descendant_of(first, second) || descendant_of(second, first)
}
