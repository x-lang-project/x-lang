use super::*;

impl TypeChecker {
    pub(in crate::expression) fn monomorphise_generic_function(
        &mut self,
        generic_name: &str,
        function: &FunctionDefinition,
        explicit_arguments: &[Type],
        actual_types: &[Type],
    ) -> Result<(String, FunctionSignature), TypeError> {
        let (substitutions, concrete_arguments) = self.infer_function_type_arguments(
            generic_name,
            function,
            explicit_arguments,
            actual_types,
        )?;
        self.validate_function_type_bounds(function, &substitutions, &concrete_arguments)?;

        let key = (generic_name.to_string(), concrete_arguments.clone());
        let existing = self.symbols.function_instantiations.get(&key).cloned();
        let concrete_name = match &existing {
            Some(name) => name.clone(),
            None => self
                .symbols
                .vacant_generated_name(&monomorphised_function_name(
                    generic_name,
                    &concrete_arguments,
                ))
                .map_err(TypeError::Invariant)?,
        };

        let signature = FunctionSignature {
            parameter_types: function
                .parameters
                .iter()
                .map(|parameter| self.substitute_type(&parameter.value_type, &substitutions))
                .collect(),
            return_type: self.substitute_type(&function.return_type, &substitutions),
            is_throws: function.is_throws,
            is_pure: function.is_pure,
            is_static: function.is_static,
        };

        if existing.is_none() {
            self.symbols
                .function_instantiations
                .insert(key, concrete_name.clone());
            self.symbols
                .add_function(concrete_name.clone(), signature.clone());
            self.symbols.function_static_parameters.insert(
                concrete_name.clone(),
                function
                    .parameters
                    .iter()
                    .map(|parameter| parameter.is_static)
                    .collect(),
            );
            self.enqueue_specialized_body(generic_name, &concrete_name, function, &substitutions);
        }

        Ok((concrete_name, signature))
    }

    fn infer_function_type_arguments(
        &mut self,
        generic_name: &str,
        function: &FunctionDefinition,
        explicit_arguments: &[Type],
        actual_types: &[Type],
    ) -> Result<(HashMap<String, Type>, Vec<Type>), TypeError> {
        for argument in explicit_arguments {
            if !is_supported_explicit_function_argument(argument) {
                return Err(TypeError::UnknownIdentifier(argument.to_string()));
            }
        }

        let accepted_explicit_arguments = function
            .generic_parameters
            .iter()
            .filter(|current_parameter| {
                !matches!(current_parameter, GenericParameter::Lifetime { .. })
            })
            .count();

        if explicit_arguments.len() > accepted_explicit_arguments {
            return Err(TypeError::ArityMismatch {
                expected: accepted_explicit_arguments,
                got: explicit_arguments.len(),
                symbol_name: generic_name.to_string(),
            });
        }

        let mut substitutions =
            explicit_type_argument_substitutions(explicit_arguments, &function.generic_parameters);

        let declared_parameters = function
            .generic_parameters
            .iter()
            .filter_map(|current_parameter| {
                if let GenericParameter::Type { name, .. } = current_parameter {
                    Some((name.clone(), Type::TypeParameter(name.clone())))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();

        for (current_parameter, actual_type) in function.parameters.iter().zip(actual_types.iter())
        {
            let pattern = self.substitute_type(&current_parameter.value_type, &declared_parameters);
            infer_type_substitutions(self, &pattern, actual_type, &mut substitutions);
        }

        for parameter in &function.generic_parameters {
            if let GenericParameter::Type {
                name,
                default: Some(default),
                ..
            } = parameter
                && !substitutions.contains_key(name)
            {
                let default = self.substitute_type(default, &substitutions);
                substitutions.insert(name.clone(), default);
            }
        }

        let concrete_arguments = function
            .generic_parameters
            .iter()
            .filter_map(|current_parameter| match current_parameter {
                GenericParameter::Type { name, .. } => Some(name),
                _ => None,
            })
            .map(|name| substitutions.get(name).cloned().unwrap_or(Type::Unknown))
            .collect::<Vec<_>>();

        if concrete_arguments.iter().any(|value_type| {
            matches!(
                value_type,
                Type::Unknown | Type::Infer(_) | Type::TypeParameter(_)
            )
        }) {
            return Err(TypeError::InferenceConflict(generic_name.to_string()));
        }

        Ok((substitutions, concrete_arguments))
    }

    fn validate_function_type_bounds(
        &mut self,
        function: &FunctionDefinition,
        substitutions: &HashMap<String, Type>,
        concrete_arguments: &[Type],
    ) -> Result<(), TypeError> {
        for (parameter, concrete_type) in function
            .generic_parameters
            .iter()
            .filter(|parameter| matches!(parameter, GenericParameter::Type { .. }))
            .zip(concrete_arguments.iter())
        {
            let GenericParameter::Type { bounds, name, .. } = parameter else {
                continue;
            };

            for bound in bounds {
                if !matches!(bound, Type::Custom(_) | Type::GenericInstance { .. }) {
                    continue;
                }

                let bound_name = self.substitute_type(bound, substitutions).to_string();

                if !self.type_implements_trait(concrete_type, &bound_name) {
                    return Err(TypeError::lookup(FrontendMessage::UnsatisfiedTraitBound {
                        concrete_type: concrete_type.to_string(),
                        name: name.to_string(),
                        bound_name: bound_name.to_string(),
                    }));
                }
            }
        }

        Ok(())
    }

    fn enqueue_specialized_body(
        &mut self,
        generic_name: &str,
        concrete_name: &str,
        function: &FunctionDefinition,
        substitutions: &HashMap<String, Type>,
    ) {
        let mut parameters = function.parameters.clone();

        for parameter in &mut parameters {
            substitute_type_in_parameter(parameter, self, substitutions);
        }

        let return_type = self.substitute_type(&function.return_type, substitutions);
        let mut body = function.body.clone();

        for statement in &mut body {
            substitute_types_in_statement(statement, self, substitutions);
        }

        let concrete_statement = Statement::Function {
            name: concrete_name.to_string(),
            generic_parameters: Vec::new(),
            parameters,
            return_type,
            body: Some(Box::new(body)),
            is_pure: function.is_pure,
            is_memoised: function.is_memoised,
            is_static: function.is_static,
            is_multi: function.is_multi,
            is_throws: function.is_throws,
            is_exported: function.is_exported,
        };

        let template_key = DeclarationKey::Function(
            generic_name.to_string(),
            function
                .parameters
                .iter()
                .map(|parameter| parameter.value_type.clone())
                .collect(),
        );

        if let (Some(origin), Some(key)) = (
            self.declaration_sources.get(&template_key).copied(),
            concrete_statement.declaration_key(),
        ) {
            self.declaration_sources.insert(key, origin);
        }

        self.pending_monomorphs.push(concrete_statement);
    }
}
