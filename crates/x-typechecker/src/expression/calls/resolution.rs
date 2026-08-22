use super::*;

impl TypeChecker {
    pub(crate) fn resolve_struct_name(&self, name: &str) -> Result<String, TypeError> {
        let is_registered = |name: &str| {
            self.symbols.records.fields().contains_key(name)
                || self.symbols.records.templates().contains_key(name)
        };

        if is_registered(name) {
            return Ok(name.to_string());
        }

        if let Some(alias) = self.symbols.imported_aliases.get(name)
            && is_registered(alias)
        {
            return Ok(alias.clone());
        }

        if let Some(resolved_namespace) = &self.current_namespace {
            let candidate = format!("{resolved_namespace}::{name}");

            if is_registered(&candidate) {
                return Ok(candidate);
            }
        }

        if let Some(current_function) = self.current_function_name()
            && let Some(namespace) = intrinsics::function_owner_name(current_function)
        {
            let candidate = format!("{namespace}::{name}");

            if is_registered(&candidate) {
                return Ok(candidate);
            }
        }

        let mut matches = self
            .symbols
            .records
            .fields()
            .keys()
            .chain(self.symbols.records.templates().keys())
            .filter(|candidate| qualified_call_path_matches(candidate, name));

        if let Some(candidate) = matches.next()
            && matches.next().is_none()
        {
            return Ok(candidate.clone());
        }

        Ok(name.to_string())
    }

    pub(in crate::expression) fn multi_candidate_matches(
        &self,
        expected: &Type,
        actual: &Type,
    ) -> bool {
        let expected = self.resolve(expected);
        let actual = self.resolve(actual);

        match (&expected, &actual) {
            (first_value, second_value) if first_value == second_value => true,
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            (Type::TypeParameter(_), _) | (_, Type::TypeParameter(_)) => true,
            (Type::Custom(first_value), Type::Custom(second_value)) => first_value == second_value,
            (
                Type::Reference {
                    is_mut: expected_mut,
                    is_unique: expected_unique,
                    inner: expected_inner,
                    ..
                },
                Type::Reference {
                    is_mut: actual_mut,
                    is_unique: actual_unique,
                    inner: actual_inner,
                    ..
                },
            ) => {
                expected_mut == actual_mut
                    && expected_unique == actual_unique
                    && self.multi_candidate_matches(expected_inner, actual_inner)
            }
            (
                Type::RawPointer {
                    is_mut: expected_mut,
                    inner: expected_inner,
                },
                Type::RawPointer {
                    is_mut: actual_mut,
                    inner: actual_inner,
                },
            ) => {
                (!expected_mut || *actual_mut)
                    && self.multi_candidate_matches(expected_inner, actual_inner)
            }
            (Type::Array(expected_inner), Type::Array(actual_inner)) => {
                self.multi_candidate_matches(expected_inner, actual_inner)
            }
            (
                Type::FixedSizedArray(expected_inner, expected_length),
                Type::FixedSizedArray(actual_inner, actual_length),
            ) => {
                expected_length == actual_length
                    && self.multi_candidate_matches(expected_inner, actual_inner)
            }
            (Type::Tuple(expected_items), Type::Tuple(actual_items)) => {
                expected_items.len() == actual_items.len()
                    && expected_items
                        .iter()
                        .zip(actual_items)
                        .all(|(expected, actual)| self.multi_candidate_matches(expected, actual))
            }
            (
                Type::GenericInstance {
                    name: expected_name,
                    type_arguments: expected_arguments,
                },
                Type::GenericInstance {
                    name: actual_name,
                    type_arguments: actual_arguments,
                },
            ) => {
                expected_name == actual_name
                    && expected_arguments.len() == actual_arguments.len()
                    && expected_arguments
                        .iter()
                        .zip(actual_arguments)
                        .all(|(expected, actual)| self.multi_candidate_matches(expected, actual))
            }
            _ => false,
        }
    }
}
