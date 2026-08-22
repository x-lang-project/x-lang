use super::*;

impl TypeChecker {
    pub(super) fn check_registered_function_call(
        &mut self,
        target: &mut x_ast::CallTarget,
        current_arguments: &mut [Expression],
        resolved_name: String,
        mut signature: FunctionSignature,
        handled: bool,
    ) -> Result<Type, TypeError> {
        if signature.is_throws && !handled {
            return Err(TypeError::inference(
                FrontendMessage::UnpropagatedThrowingCall {
                    resolved_name: resolved_name.to_string(),
                },
            ));
        }

        if let Some(arguments) = target.owner_type_arguments()
            && let Some(owner) = intrinsics::function_owner_name(&resolved_name)
            && let Some(key) = self.resolve_generic_struct_key(owner)
            && let Some(definition) = self.symbols.records.templates().get(&key).cloned()
        {
            let parameters = definition
                .generic_parameters
                .as_ref()
                .into_iter()
                .flatten()
                .filter_map(x_ast::GenericParameter::non_lifetime_name);
            let parameters = parameters.cloned().collect::<Vec<_>>();

            if parameters.len() != arguments.len() {
                return Err(TypeError::ArityMismatch {
                    expected: parameters.len(),
                    got: arguments.len(),
                    symbol_name: format!("{owner} type arguments"),
                });
            }

            let substitutions = parameters
                .into_iter()
                .zip(arguments.iter().cloned())
                .collect::<HashMap<_, _>>();
            signature.parameter_types = signature
                .parameter_types
                .iter()
                .map(|value_type| self.substitute_type(value_type, &substitutions))
                .collect();
            signature.return_type = self.substitute_type(&signature.return_type, &substitutions);
        }

        if current_arguments.len() != signature.parameter_types.len() {
            return Err(TypeError::ArityMismatch {
                expected: signature.parameter_types.len(),
                got: current_arguments.len(),
                symbol_name: resolved_name,
            });
        }

        self.check_pure_call(&resolved_name, &signature)?;

        let mut actual_types = Vec::with_capacity(current_arguments.len());

        for (argument, expected_type) in
            current_arguments.iter_mut().zip(&signature.parameter_types)
        {
            let mut actual_type = self.check_expression(argument)?;

            if is_integer_literal(argument) && self.is_integer_primitive(expected_type) {
                argument.value_type = expected_type.clone();
                actual_type = expected_type.clone();
            } else {
                argument.value_type = actual_type.clone();
            }

            actual_types.push(actual_type);
        }
        signature = call_lifetimes::instantiate(&signature, &actual_types)?;
        for (index, ((argument, expected_type), actual_type)) in current_arguments
            .iter()
            .zip(&signature.parameter_types)
            .zip(&actual_types)
            .enumerate()
        {
            self.unify(expected_type, actual_type).map_err(|error| {
                error
                    .map_expected(|expected| {
                        format!(
                            "{expected} for argument {} of '{}'",
                            index + 1,
                            resolved_name
                        )
                    })
                    .at_source(argument.span)
            })?;
        }

        target.resolve_to(resolved_name);

        let return_type = self.resolve_type(&signature.return_type)?;

        Ok(return_type)
    }
}
