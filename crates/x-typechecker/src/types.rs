mod substitution;

mod compatibility;

use std::collections::HashMap;

use crate::TypeAliasDefinition;
use crate::TypeChecker;
use crate::tail_helpers::qualified_path_suffix_matches;
use x_ast::Type;

pub(crate) fn lifetimes_compatible(
    expected: &Option<x_ast::Lifetime>,
    actual: &Option<x_ast::Lifetime>,
) -> bool {
    use x_ast::Lifetime;

    match (expected, actual) {
        (None, _) | (_, None) => true,
        (Some(Lifetime::Erased), _) | (_, Some(Lifetime::Erased)) => true,
        (Some(_), Some(Lifetime::Static)) => true,
        (Some(Lifetime::Named(expected)), Some(Lifetime::Named(actual))) => expected == actual,
        (Some(Lifetime::Infer(_)), _) | (_, Some(Lifetime::Infer(_))) => true,
        _ => false,
    }
}
