use super::*;

impl TypeChecker {
    pub fn synthesise_default_methods(
        &mut self,
        statements: &mut [Statement],
    ) -> Result<(), errors::TypeError> {
        let impls = self.symbols.impls_for_type.clone();
        let mut synthesis_plan = Vec::new();
        // Preserve declaration order for trait aliases and duplicate declarations.
        let mut impl_index: HashMap<Type, Vec<usize>> = HashMap::new();

        for (index, statement) in statements.iter().enumerate() {
            if let Statement::ImplDeclaration(implementation) = statement {
                impl_index
                    .entry(implementation.type_name.clone())
                    .or_default()
                    .push(index);
            }
        }

        for (impl_type, trait_names) in &impls {
            let struct_name = match &impl_type {
                Type::Custom(name) | Type::GenericInstance { name, .. } => name.clone(),
                Type::TypeParameter(name) => name.clone(),
                other => other.to_string(),
            };

            for trait_name in trait_names {
                let trait_key = self.resolve_trait_key(trait_name)?;

                let Some(trait_key) = trait_key else {
                    continue;
                };

                let Some(trait_definition) =
                    self.symbols.trait_ast_definitions.get(&trait_key).cloned()
                else {
                    continue;
                };

                let trait_type_map =
                    self.bind_trait_type_arguments(&trait_definition, trait_name, impl_type)?;

                for (method_name, method_statement) in trait_definition.methods {
                    let Some(mut method_statement) = method_statement else {
                        continue;
                    };

                    if !trait_method_has_default_body(&method_statement) {
                        continue;
                    }

                    let has_impl_method = impl_index
                        .get(impl_type)
                        .into_iter()
                        .flatten()
                        .filter_map(|index| statements.get(*index))
                        .any(|statement| {
                            let Statement::ImplDeclaration(implementation) = statement else {
                                return false;
                            };

                            self.same_trait_instance(&implementation.trait_name, trait_name, impl_type)
                                && implementation.methods.iter().any(|method| {
                                    matches!(method, Statement::Function { name, .. } if name == &method_name)
                                })
                        });

                    if self
                        .symbols
                        .struct_methods
                        .get(&struct_name)
                        .is_some_and(|methods| methods.contains_key(&method_name))
                        || has_impl_method
                    {
                        continue;
                    }

                    substitute_self_in_statement(&mut method_statement, impl_type);
                    substitute_trait_method_types(&mut method_statement, self, &trait_type_map);

                    let Statement::Function {
                        parameters,
                        return_type,
                        is_pure,
                        is_throws,
                        is_static,
                        ..
                    } = &method_statement
                    else {
                        continue;
                    };

                    self.symbols.add_struct_method(
                        struct_name.clone(),
                        method_name.clone(),
                        crate::typed_function_signature!(
                            parameters,
                            return_type,
                            *is_pure,
                            *is_throws,
                            *is_static
                        ),
                    );
                    synthesis_plan.push((impl_type.clone(), trait_name.clone(), method_statement));
                }
            }
        }

        for (impl_type, trait_name, method_statement) in synthesis_plan {
            let position = impl_index
                .get(&impl_type)
                .into_iter()
                .flatten()
                .copied()
                .find(|index| {
                    matches!(statements.get(*index), Some(Statement::ImplDeclaration(implementation))
                        if self.same_trait_instance(&implementation.trait_name, &trait_name, &impl_type))
                });

            if let Some(Statement::ImplDeclaration(implementation)) =
                position.and_then(|index| statements.get_mut(index))
            {
                implementation.methods.push(method_statement);
            }
        }

        Ok(())
    }
}
