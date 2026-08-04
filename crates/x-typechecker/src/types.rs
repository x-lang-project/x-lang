mod substitution;

mod compatibility;

use std::collections::HashMap;

use crate::TypeAliasDefinition;
use crate::TypeChecker;
use crate::tail_helpers::qualified_path_suffix_matches;
use x_ast::Type;

impl TypeChecker {
    pub(crate) fn lookup_type_alias(&self, name: &str) -> Option<TypeAliasDefinition> {
        self.symbols
            .imported_aliases
            .get(name)
            .and_then(|key| self.symbols.type_aliases.get(key))
            .cloned()
            .or_else(|| self.symbols.type_aliases.get(name).cloned())
            .or_else(|| {
                self.current_namespace.as_ref().and_then(|namespace| {
                    self.symbols
                        .type_aliases
                        .get(&format!("{namespace}::{name}"))
                        .cloned()
                })
            })
            .or_else(|| {
                let mut matches = self
                    .symbols
                    .type_aliases
                    .iter()
                    .filter(|(alias_name, _)| qualified_alias_name_matches(alias_name, name));

                let first = matches.next()?;

                if matches.next().is_none() {
                    Some(first.1.clone())
                } else {
                    None
                }
            })
    }
}

fn qualified_alias_name_matches(registered: &str, requested: &str) -> bool {
    qualified_path_suffix_matches(registered, requested)
}

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
