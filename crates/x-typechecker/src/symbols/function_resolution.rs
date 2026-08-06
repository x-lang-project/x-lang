use std::sync::Arc;

use super::{FunctionDefinition, MultiFunctionCandidate, SymbolTable};
use crate::helpers::strip_generic_arguments_from_path;
use crate::tail_helpers::qualified_path_suffix_matches;

impl SymbolTable {
    fn resolve_qualified_function_name(&self, name: &str) -> Option<String> {
        if self.functions.contains_key(name) {
            return Some(name.to_owned());
        }

        if let Some(alias) = self.imported_aliases.get(name) {
            return self.functions.contains_key(alias).then(|| alias.clone());
        }

        if let Ok(cache) = self.qualified_function_cache.read()
            && let Some(resolved) = cache.get(name)
        {
            return resolved.clone();
        }

        let resolved = self
            .imported_aliases
            .get(name)
            .filter(|key| self.functions.contains_key(*key))
            .cloned();

        if name.len() <= 4096
            && resolved.as_ref().is_none_or(|key| key.len() <= 4096)
            && let Ok(mut cache) = self.qualified_function_cache.write()
            && cache.len() < 256
        {
            cache.insert(name.to_owned(), resolved.clone());
        }

        resolved
    }
}
