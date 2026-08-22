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
    pub(super) fn supertrait_associated_types(
        &mut self,
        definition: &TraitDefinition,
        target: &Type,
        substitutions: &HashMap<String, Type>,
        hierarchy: &mut TraitHierarchy<'_>,
    ) -> Result<BTreeMap<String, Type>, errors::TypeError> {
        let mut result = BTreeMap::new();

        for bound in &definition.supertraits {
            let mut bound = self.substitute_type(bound, substitutions);
            substitute_self_in_type(&mut bound, target);
            let name = bound.to_string();
            let Some(instance) = self.resolved_trait_instance(&name, target)? else {
                return Err(errors::TypeError::lookup(FrontendMessage::UnknownTrait {
                    trait_name: name,
                }));
            };

            if hierarchy.visiting.contains(&instance.key) {
                return Err(errors::TypeError::lookup(FrontendMessage::UnknownTrait {
                    trait_name: format!("cyclic supertrait {}", instance.key),
                }));
            }

            let statements = hierarchy.statements;
            let implementation = statements.iter().find_map(|statement| {
                let Statement::ImplDeclaration(implementation) = statement else {
                    return None;
                };

                (implementation.type_name == *target
                    && self.same_trait_instance(&implementation.trait_name, &name, target))
                .then_some(implementation)
            });
            let Some(implementation) = implementation else {
                return Err(errors::TypeError::lookup(FrontendMessage::UnknownTrait {
                    trait_name: format!("{name} implementation for {target}"),
                }));
            };

            let parent = self
                .symbols
                .trait_ast_definitions
                .get(&instance.key)
                .cloned()
                .ok_or_else(|| {
                    errors::TypeError::lookup(FrontendMessage::UnknownTrait {
                        trait_name: instance.key.clone(),
                    })
                })?;
            let arguments = instance.arguments.into_iter().collect();
            hierarchy.visiting.push(instance.key);
            let inherited =
                self.supertrait_associated_types(&parent, target, &arguments, hierarchy);
            hierarchy.visiting.pop();
            let inherited = inherited?;

            for (key, value) in
                inherited
                    .into_iter()
                    .chain(parent.associated_types.iter().filter_map(|(key, default)| {
                        implementation
                            .associated_types
                            .get(key)
                            .cloned()
                            .or_else(|| (*default != Type::Unknown).then(|| default.clone()))
                            .map(|value| (key.clone(), value))
                    }))
            {
                let value = self.substitute_type(&value, &arguments);

                if let Some(previous) = result.insert(key.clone(), value.clone())
                    && previous != value
                {
                    return Err(crate::type_mismatch!(
                        previous.to_string(),
                        format!("{value} for ambiguous associated type {key}")
                    ));
                }
            }
        }

        Ok(result)
    }

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

    pub(super) fn resolved_trait_instance(
        &mut self,
        trait_name: &str,
        target: &Type,
    ) -> Result<Option<ResolvedTraitInstance>, errors::TypeError> {
        let Some(key) = self.resolve_trait_key(trait_name)? else {
            return Ok(None);
        };

        let Some(definition) = self.symbols.trait_ast_definitions.get(&key).cloned() else {
            return Ok(None);
        };

        let arguments = self.bind_trait_type_arguments(&definition, trait_name, target)?;
        let arguments = arguments
            .into_iter()
            .map(|(name, argument)| {
                self.resolve_type(&argument)
                    .map(|argument| (name, argument))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;

        Ok(Some(ResolvedTraitInstance { key, arguments }))
    }

    pub(super) fn same_trait_instance(&mut self, left: &str, right: &str, target: &Type) -> bool {
        let Ok(Some(left)) = self.resolved_trait_instance(left, target) else {
            return false;
        };

        self.resolved_trait_instance(right, target)
            .ok()
            .flatten()
            .as_ref()
            == Some(&left)
    }

    pub(crate) fn type_implements_trait(&mut self, value_type: &Type, trait_name: &str) -> bool {
        let traits = self
            .symbols
            .impls_for_type
            .iter()
            .filter(|(impl_type, _)| type_trait_target_matches(impl_type, value_type))
            .flat_map(|(_, traits)| traits.clone())
            .collect::<Vec<_>>();

        traits
            .iter()
            .any(|implemented| self.same_trait_instance(implemented, trait_name, value_type))
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
