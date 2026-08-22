use super::*;

impl TypeChecker {
    pub(super) fn index_global_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<(), TypeError> {
        match statement {
            Statement::VariableDeclaration {
                pattern: Pattern::Identifier(name),
                type_annotation,
                ..
            } => {
                let value_type = type_annotation.clone().unwrap_or(Type::Unknown);
                self.symbols.add_variable(name.clone(), value_type);
            }
            Statement::Constant {
                name,
                type_annotation,
                ..
            } => {
                let value_type = type_annotation.clone().unwrap_or(Type::Unknown);
                self.symbols.add_global_constant(name.clone(), value_type);
            }
            Statement::TypeAlias {
                name,
                generic_parameters,
                value_type,
            } => {
                let fully_qualified = self.qualify_indexed_name(name);

                self.symbols.type_aliases.insert(
                    fully_qualified,
                    TypeAliasDefinition {
                        generic_parameters: generic_parameters.clone(),
                        value_type: value_type.clone(),
                    },
                );
            }
            Statement::Function {
                name,
                parameters,
                return_type,
                generic_parameters,
                is_pure,
                is_throws,
                is_static,
                is_multi,
                body,
                is_memoised,
                is_exported,
                ..
            } => {
                let fully_qualified = self.qualify_indexed_name(name);

                if *is_multi
                    && generic_parameters.iter().any(|parameter| {
                        !matches!(parameter, x_ast::GenericParameter::Lifetime { .. })
                    })
                {
                    return Err(TypeError::invalid(
                        FrontendMessage::UnsupportedMultiDeclaration {
                            name: fully_qualified,
                        },
                    ));
                }

                for parameter in parameters {
                    let resolved = self.resolve_type(&parameter.value_type)?;

                    if *is_pure && self.type_invalidates_pure_memory_contract(&resolved) {
                        return Err(TypeError::invalid(FrontendMessage::MutablePureParameter {
                            fully_qualified: fully_qualified.to_string(),
                            name: parameter.name.to_string(),
                        }));
                    }

                    if parameter.is_static && type_contains_runtime_handle(&resolved) {
                        return Err(TypeError::invalid(
                            FrontendMessage::RuntimeHandleStaticParameter {
                                name: parameter.name.to_string(),
                                fully_qualified: fully_qualified.to_string(),
                            },
                        ));
                    }
                }

                if generic_parameters
                    .iter()
                    .any(|parameter| !matches!(parameter, x_ast::GenericParameter::Lifetime { .. }))
                {
                    let function = FunctionDefinition {
                        name: fully_qualified.clone(),
                        generic_parameters: generic_parameters.clone(),
                        parameters: parameters.clone(),
                        return_type: return_type.clone(),
                        body: body
                            .as_ref()
                            .map(|second_value| *second_value.clone())
                            .unwrap_or_default(),
                        is_exported: *is_exported,
                        is_pure: *is_pure,
                        is_memoised: *is_memoised,
                        is_static: *is_static,
                        is_multi: *is_multi,
                        is_throws: *is_throws,
                    };
                    self.symbols
                        .generic_functions
                        .insert(fully_qualified, function.into());

                    return Ok(());
                }

                let signature = crate::typed_function_signature!(
                    parameters,
                    return_type,
                    *is_pure,
                    *is_throws,
                    *is_static,
                );

                if *is_multi {
                    self.index_multi_function(&fully_qualified, signature)?;

                    if let Some(candidate) = self
                        .symbols
                        .multi_functions
                        .get(&fully_qualified)
                        .and_then(|candidates| candidates.last())
                    {
                        self.symbols.function_static_parameters.insert(
                            candidate.mangled_name.clone(),
                            parameters
                                .iter()
                                .map(|parameter| parameter.is_static)
                                .collect(),
                        );
                    }
                } else {
                    self.symbols.function_static_parameters.insert(
                        fully_qualified.clone(),
                        parameters
                            .iter()
                            .map(|parameter| parameter.is_static)
                            .collect(),
                    );
                    self.symbols.add_function(fully_qualified, signature);
                }
            }

            Statement::StructDeclaration(sdef) => self.index_struct_declaration(sdef),
            Statement::TraitDeclaration(trait_definition) => {
                self.index_trait_declaration(trait_definition)?
            }
            Statement::ImplDeclaration(impl_definition) => {
                self.index_impl_declaration(impl_definition)?
            }
            Statement::ExternFunctionDeclaration {
                name,
                parameters,
                return_type,
                abi,
                is_pure,
                ..
            } => self.index_external_declaration(ExternDeclaration {
                name,
                parameters,
                return_type: return_type.as_deref(),
                abi: abi.as_deref(),
                is_pure: *is_pure,
            })?,
            _ => {}
        }

        Ok(())
    }
}
