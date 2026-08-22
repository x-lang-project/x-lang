use super::*;

impl TypeChecker {
    pub(super) fn index_struct_declaration(&mut self, sdef: &x_ast::StructDefinition) {
        let mut sdef = sdef.clone();
        sdef.name = self.qualify_indexed_name(&sdef.name);
        if let Some(invariant) = &sdef.invariant {
            self.register_struct_invariant(sdef.name.clone(), invariant);
        }

        self.symbols.records.index_declaration(sdef);
    }

    pub(super) fn index_trait_declaration(
        &mut self,
        trait_definition: &x_ast::TraitDefinition,
    ) -> Result<(), TypeError> {
        let trait_name = self.qualify_indexed_name(&trait_definition.name);
        let mut methods = HashMap::new();

        for (method_name, method_statement) in &trait_definition.methods {
            let Some(Statement::Function {
                parameters,
                return_type,
                is_pure,
                is_throws,
                is_static,
                is_multi,
                ..
            }) = method_statement
            else {
                continue;
            };

            if *is_multi {
                return Err(TypeError::invalid(
                    FrontendMessage::UnsupportedMultiDeclaration {
                        name: format!("{trait_name}::{method_name}"),
                    },
                ));
            }

            methods.insert(
                method_name.clone(),
                crate::typed_function_signature!(
                    parameters,
                    return_type,
                    *is_pure,
                    *is_throws,
                    *is_static,
                ),
            );
        }

        self.symbols.traits.insert(
            trait_name.clone(),
            TraitInfo {
                name: trait_name.clone(),
                methods,
            },
        );
        let mut qualified_trait = trait_definition.clone();
        qualified_trait.name = trait_name.clone();
        self.symbols
            .trait_ast_definitions
            .insert(trait_name, qualified_trait);

        Ok(())
    }

    pub(super) fn index_impl_declaration(
        &mut self,
        impl_definition: &x_ast::ImplDefinition,
    ) -> Result<(), TypeError> {
        let mut struct_name = match &impl_definition.type_name {
            Type::Custom(name) => name.clone(),
            Type::GenericInstance { .. } => format!("{}", impl_definition.type_name),
            Type::TypeParameter(name) => name.clone(),
            other => other.to_string(),
        };

        struct_name = self.qualify_indexed_name(&struct_name);

        if !impl_definition.trait_name.is_empty() {
            self.symbols
                .impls_for_type
                .entry(impl_definition.type_name.clone())
                .or_default()
                .push(impl_definition.trait_name.clone());
        }

        for method in &impl_definition.methods {
            let Statement::Function {
                name,
                parameters,
                return_type,
                generic_parameters,
                is_pure,
                is_throws,
                is_static,
                is_memoised,
                is_multi,
                is_exported,
                body,
                ..
            } = method
            else {
                continue;
            };

            let signature = crate::typed_function_signature!(
                parameters,
                return_type,
                *is_pure,
                *is_throws,
                *is_static,
            );
            let qualified_name = format!("{struct_name}::{name}");

            if *is_multi {
                return Err(TypeError::invalid(
                    FrontendMessage::UnsupportedMultiDeclaration {
                        name: qualified_name,
                    },
                ));
            }

            if parameters
                .first()
                .is_some_and(x_ast::Parameter::is_receiver)
            {
                self.symbols
                    .add_struct_method(struct_name.clone(), name.clone(), signature);
            } else if generic_parameters
                .iter()
                .any(|parameter| !matches!(parameter, x_ast::GenericParameter::Lifetime { .. }))
            {
                self.symbols.generic_functions.insert(
                    qualified_name.clone(),
                    FunctionDefinition {
                        name: qualified_name,
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
                    }
                    .into(),
                );
            } else if *is_multi {
                self.index_multi_function(&qualified_name, signature)?;
            } else {
                self.symbols.add_function(qualified_name, signature);
            }
        }

        Ok(())
    }

    pub(super) fn index_external_declaration(
        &mut self,
        declaration: ExternDeclaration<'_>,
    ) -> Result<(), TypeError> {
        let ExternDeclaration {
            name,
            parameters,
            return_type,
            abi,
            is_pure,
        } = declaration;

        validate_external_c_abi(name, abi)?;
        let parameter_types: Vec<Type> = parameters
            .iter()
            .map(|pointer| Type::from_external_name(&pointer.type_name))
            .collect();

        for (index, value_type) in parameter_types.iter().enumerate() {
            if !external_ffi_type_is_supported(value_type, false) {
                return Err(TypeError::inference(
                    FrontendMessage::UnsupportedExternParameter {
                        name: name.to_string(),
                        parameter_index: index + 1,
                        value_type: value_type.to_string(),
                    },
                ));
            }
        }

        let current_return_type = return_type
            .map(Type::from_external_name)
            .unwrap_or(Type::Void);

        if !external_ffi_type_is_supported(&current_return_type, true) {
            return Err(TypeError::inference(
                FrontendMessage::UnsupportedExternReturn {
                    name: name.to_string(),
                    return_type: current_return_type.to_string(),
                },
            ));
        }

        if is_pure {
            return Err(TypeError::invalid(
                FrontendMessage::UninspectablePureExtern {
                    name: name.to_string(),
                },
            ));
        }

        self.symbols.add_function(
            name.to_owned(),
            FunctionSignature {
                parameter_types,
                return_type: current_return_type,
                is_throws: false,
                is_pure: false,
                is_static: false,
            },
        );

        Ok(())
    }
}
