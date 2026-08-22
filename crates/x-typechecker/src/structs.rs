mod invariants;

use std::collections::HashMap;

use crate::TypeChecker;
use crate::errors;
use crate::expression;
use crate::tail_helpers::qualified_path_suffix_matches;
use x_ast::DeclarationKey;
use x_ast::{Statement, StructDefinition, Type};
use x_diagnostics::frontend_messages::FrontendIssue;

impl TypeChecker {
    pub(crate) fn resolve_generic_struct_key(&self, name: &str) -> Option<String> {
        if let Some(alias) = self.symbols.imported_aliases.get(name)
            && self.symbols.records.templates().contains_key(alias)
        {
            return Some(alias.clone());
        }

        if self.symbols.records.templates().contains_key(name) {
            return Some(name.to_string());
        }

        let mut matches = self
            .symbols
            .records
            .templates()
            .keys()
            .filter(|candidate| qualified_name_matches(candidate, name));

        let first = matches.next()?.clone();

        if matches.next().is_none() {
            Some(first)
        } else {
            None
        }
    }
}

fn qualified_name_matches(registered: &str, requested: &str) -> bool {
    if registered == requested {
        return true;
    }

    qualified_path_suffix_matches(registered, requested)
}
