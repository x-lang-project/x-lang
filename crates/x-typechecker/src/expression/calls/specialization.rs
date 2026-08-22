use super::*;

impl TypeChecker {
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
