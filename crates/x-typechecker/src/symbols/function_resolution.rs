use std::sync::Arc;

use super::{FunctionDefinition, MultiFunctionCandidate, SymbolTable};
use crate::helpers::strip_generic_arguments_from_path;
use crate::tail_helpers::qualified_path_suffix_matches;

impl SymbolTable {
    pub(crate) fn resolve_function_call_name(
        &self,
        name: &str,
        base_name: &str,
        current_function: Option<&str>,
    ) -> String {
        if self.functions.contains_key(name) {
            return name.to_owned();
        }

        let stripped_name = strip_generic_arguments_from_path(name);

        if self.functions.contains_key(&stripped_name) {
            return stripped_name;
        }

        let resolved = if name.contains("::") {
            self.resolve_qualified_function_name(&stripped_name)
        } else {
            self.resolve_unqualified_function_name(base_name, current_function)
        };

        resolved.unwrap_or_else(|| name.to_owned())
    }

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

    fn resolve_unqualified_function_name(
        &self,
        base_name: &str,
        current_function: Option<&str>,
    ) -> Option<String> {
        if let Some(alias) = self.imported_aliases.get(base_name)
            && self.functions.contains_key(alias)
        {
            return Some(alias.clone());
        }

        let stripped_base = base_name.split('<').next().unwrap_or(base_name);

        if let Some(namespace) = current_function.and_then(x_ast::intrinsics::function_owner_name) {
            let candidate = format!("{namespace}::{stripped_base}");

            if self.functions.contains_key(&candidate) {
                return Some(candidate);
            }
        }

        let suffix = format!("::{stripped_base}");
        let mut matches = self.functions.keys().filter(|key| {
            key.ends_with(&suffix)
                && !self.qualified_imports.contains(*key)
                && !x_ast::intrinsics::function_owner_name(key).is_some_and(|owner| {
                    self.records.fields().contains_key(owner)
                        || self.records.templates().contains_key(owner)
                        || self.struct_methods.contains_key(owner)
                })
        });
        let only = matches.next()?;

        matches.next().is_none().then(|| only.clone())
    }

    pub(crate) fn resolve_multi_candidates(
        &self,
        name: &str,
        base_name: &str,
    ) -> Option<(String, Vec<MultiFunctionCandidate>)> {
        let normalised_name = strip_generic_arguments_from_path(name);
        let stripped_name = normalised_name.as_str();
        let stripped_base = base_name.split('<').next().unwrap_or(base_name);

        for key in [name, stripped_name] {
            if let Some(candidates) = self.multi_functions.get(key) {
                return Some((key.to_string(), candidates.clone()));
            }
        }

        let requested = if name.contains("::") {
            stripped_name
        } else {
            stripped_base
        };

        let mut matches = self.multi_functions.iter().filter(|(key, _)| {
            (!name.contains("::") || key.contains("::"))
                && (name.contains("::") || !self.qualified_imports.contains(*key))
                && qualified_call_path_matches(key, requested)
        });
        let (key, candidates) = matches.next()?;

        if matches.next().is_some() {
            return None;
        }

        Some((key.clone(), candidates.clone()))
    }

    pub(crate) fn resolve_generic_function(
        &self,
        name: &str,
        base_name: &str,
    ) -> Option<(String, Arc<FunctionDefinition>)> {
        let stripped_base = base_name.split('<').next().unwrap_or(base_name);
        let normalised_name = strip_generic_arguments_from_path(name);

        if let Some(callee) = self.generic_functions.get(name) {
            return Some((name.to_string(), callee.clone()));
        }

        if let Some(callee) = self.generic_functions.get(&normalised_name) {
            return Some((normalised_name, callee.clone()));
        }

        let suffix = format!("::{stripped_base}");

        let requested = self
            .imported_aliases
            .get(&normalised_name)
            .map(String::as_str)
            .unwrap_or(&normalised_name);
        let mut matches = self.generic_functions.iter().filter(|(key, _)| {
            if name.contains("::") {
                key.as_str() == requested
            } else {
                (!self.qualified_imports.contains(*key) && key.ends_with(&suffix))
                    || key.as_str() == requested
            }
        });
        let (key, function) = matches.next()?;

        matches
            .next()
            .is_none()
            .then(|| (key.clone(), function.clone()))
    }
}

pub(crate) fn qualified_call_path_matches(registered: &str, requested: &str) -> bool {
    if registered == requested {
        return true;
    }

    let registered_normalised = strip_generic_arguments_from_path(registered);
    let requested_normalised = strip_generic_arguments_from_path(requested);
    let registered = registered_normalised.as_str();
    let requested = requested_normalised.as_str();

    qualified_path_suffix_matches(registered, requested)
}
