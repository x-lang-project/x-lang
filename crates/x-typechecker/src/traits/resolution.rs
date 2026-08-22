use super::*;

pub(super) struct TraitHierarchy<'a> {
    statements: &'a [Statement],
    visiting: Vec<String>,
}

impl<'a> TraitHierarchy<'a> {
    pub(super) fn new(statements: &'a [Statement]) -> Self {
        Self {
            statements,
            visiting: Vec::new(),
        }
    }
}

impl TypeChecker {
    pub(super) fn bind_trait_type_arguments(
        &mut self,
        trait_definition: &TraitDefinition,
        trait_name: &str,
        self_type: &Type,
    ) -> Result<HashMap<String, Type>, errors::TypeError> {
        let arguments = trait_generic_arguments_for_impl(trait_name);
        let parameters = trait_definition
            .generic_parameters
            .iter()
            .filter_map(|parameter| match parameter {
                x_ast::GenericParameter::Type { name, default, .. } => Some((name, default)),
                _ => None,
            })
            .collect::<Vec<_>>();

        if arguments.len() > parameters.len() {
            return Err(errors::TypeError::ArityMismatch {
                expected: parameters.len(),
                got: arguments.len(),
                symbol_name: trait_name.into(),
            });
        }

        let mut substitutions = HashMap::new();

        for (index, (name, default)) in parameters.into_iter().enumerate() {
            let mut argument = match arguments.get(index) {
                Some(argument) => argument.clone(),
                None => default
                    .clone()
                    .ok_or_else(|| errors::TypeError::ArityMismatch {
                        expected: trait_generic_parameters(trait_definition).len(),
                        got: arguments.len(),
                        symbol_name: trait_name.into(),
                    })?,
            };

            substitute_self_in_type(&mut argument, self_type);
            argument = self.substitute_type(&argument, &substitutions);
            substitutions.insert(name.clone(), argument);
        }

        Ok(substitutions)
    }

    pub(super) fn resolve_trait_key(
        &self,
        trait_name: &str,
    ) -> Result<Option<String>, errors::TypeError> {
        let normalised = normalise_trait_name(trait_name);

        if self.symbols.trait_ast_definitions.contains_key(&normalised) {
            return Ok(Some(normalised));
        }

        if let Some(alias) = self.symbols.imported_aliases.get(&normalised)
            && self.symbols.trait_ast_definitions.contains_key(alias)
        {
            return Ok(Some(alias.clone()));
        }

        if !normalised.contains("::")
            && let Some(namespace) = &self.current_namespace
        {
            let candidate = format!("{namespace}::{normalised}");

            if self.symbols.trait_ast_definitions.contains_key(&candidate) {
                return Ok(Some(candidate));
            }
        }

        if normalised.contains("::") {
            return Ok(None);
        }

        let mut candidates = self
            .symbols
            .trait_ast_definitions
            .keys()
            .filter(|key| trait_names_match(key, trait_name))
            .collect::<Vec<_>>();
        candidates.sort();

        match candidates.as_slice() {
            [] => Ok(None),
            [name] => Ok(Some((*name).clone())),
            _ => Err(errors::TypeError::inference(
                FrontendMessage::AmbiguousTrait {
                    trait_name: trait_name.to_string(),
                    candidates: (candidates
                        .iter()
                        .map(|name| name.as_str())
                        .collect::<Vec<_>>()
                        .join(", "))
                    .to_string(),
                },
            )),
        }
    }
}
