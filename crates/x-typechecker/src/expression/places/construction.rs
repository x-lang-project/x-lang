use super::*;

impl TypeChecker {
    pub(in crate::expression) fn check_struct_instantiation(
        &mut self,
        initialise: &mut StructInitialisation,
        expected_type: &Type,
    ) -> Result<Type, TypeError> {
        let mut struct_name = self.resolve_struct_name(&initialise.name)?;

        if let Type::Custom(expected_name) = expected_type
            && self
                .symbols
                .records
                .instantiations()
                .iter()
                .any(|((origin, _), concrete)| origin == &struct_name && concrete == expected_name)
        {
            struct_name = expected_name.clone();
        }

        let generic_definition = self.symbols.records.templates().get(&struct_name).cloned();
        let mut generic_bindings = HashMap::new();
        let declared = generic_definition
            .as_ref()
            .and_then(|definition| definition.generic_parameters.clone())
            .unwrap_or_default()
            .iter()
            .filter_map(x_ast::GenericParameter::non_lifetime_name)
            .cloned()
            .collect::<Vec<_>>();

        let layout = if let Some(layout) = self.symbols.records.fields().get(&struct_name).cloned()
        {
            layout
        } else if let Some(sdef) = self.symbols.records.templates().get(&struct_name) {
            sdef.fields.clone()
        } else {
            return Err(TypeError::lookup(FrontendMessage::UnknownStruct {
                name: initialise.name.to_string(),
            }));
        };

        let expected = layout
            .iter()
            .map(|(name, value_type)| (name.as_str(), value_type))
            .collect::<HashMap<_, _>>();

        let mut seen = HashSet::new();
        let mut checked_fields = HashMap::new();

        for (field_name, field_expression) in initialise.fields.iter_mut() {
            let Some(current_expected_type) = expected.get(field_name.as_str()) else {
                return Err(TypeError::UnresolvedProperty {
                    field: field_name.clone(),
                    value_type: Type::Custom(struct_name.clone()),
                });
            };

            let got_type = self.check_expression(field_expression)?;

            if generic_definition.is_some() {
                crate::expression::generic_types::infer_declared_type_substitutions(
                    self,
                    current_expected_type,
                    &got_type,
                    &declared,
                    &mut generic_bindings,
                );
                field_expression.value_type = got_type;
            } else {
                self.unify(current_expected_type, &got_type)?;
                field_expression.value_type = self.resolve(current_expected_type);
            }

            if !seen.insert(field_name.clone()) {
                return Err(TypeError::lookup(
                    FrontendMessage::DuplicateFieldInitializer {
                        field_name: field_name.to_string(),
                        struct_name: struct_name.to_string(),
                    },
                ));
            }

            checked_fields.insert(field_name.clone(), field_expression.clone());
        }

        if let Some((missing_field, _)) = layout
            .iter()
            .find(|(field_name, _)| !seen.contains(field_name))
        {
            return Err(TypeError::UnresolvedProperty {
                field: missing_field.clone(),
                value_type: Type::Custom(struct_name.clone()),
            });
        }

        if let Some(definition) = &generic_definition {
            let concrete_arguments = definition
                .generic_parameters
                .as_ref()
                .into_iter()
                .flatten()
                .filter_map(x_ast::GenericParameter::non_lifetime_name)
                .map(|parameter| {
                    generic_bindings
                        .get(parameter)
                        .map(|value_type| self.resolve(value_type))
                        .ok_or_else(|| {
                            TypeError::inference(FrontendMessage::MissingDeclaredTypeArgument {
                                parameter: parameter.clone(),
                                owner: struct_name.clone(),
                            })
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;

            if concrete_arguments.iter().any(|value_type| {
                matches!(
                    value_type,
                    Type::Unknown | Type::Infer(_) | Type::TypeParameter(_)
                )
            }) {
                return Err(TypeError::InferenceConflict(struct_name));
            }

            struct_name = self.instantiate_struct(&struct_name, &concrete_arguments)?;
            let concrete_layout = self
                .symbols
                .struct_fields(&struct_name)
                .cloned()
                .ok_or_else(|| {
                    TypeError::inference(FrontendMessage::UnknownStruct {
                        name: struct_name.clone(),
                    })
                })?;

            for (field, current_expected_type) in concrete_layout {
                let expression = checked_fields.get_mut(&field).ok_or_else(|| {
                    TypeError::UnresolvedProperty {
                        field: field.clone(),
                        value_type: Type::Custom(struct_name.clone()),
                    }
                })?;
                let current_expected_type = self.resolve_type(&current_expected_type)?;
                self.unify(&current_expected_type, &expression.value_type)?;
                expression.value_type = self.resolve(&current_expected_type);
            }
        }
        initialise.name = struct_name.clone();
        initialise.fields = layout
            .iter()
            .filter_map(|(field_name, _)| {
                checked_fields
                    .remove(field_name)
                    .map(|inner_expression| (field_name.clone(), inner_expression))
            })
            .collect();

        Ok(Type::Custom(struct_name))
    }
}
