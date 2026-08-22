use super::*;

impl TypeChecker {
    pub(super) fn check_generic_function_call(
        &mut self,
        target: &mut x_ast::CallTarget,
        arguments: &mut [Expression],
        generic_name: String,
        generic_function: &FunctionDefinition,
        handled: bool,
    ) -> Result<Type, TypeError> {
        if arguments.len() != generic_function.parameters.len() {
            return Err(TypeError::ArityMismatch {
                expected: generic_function.parameters.len(),
                got: arguments.len(),
                symbol_name: generic_name,
            });
        }

        let mut actual_types = Vec::with_capacity(arguments.len());

        for argument in arguments.iter_mut() {
            let actual_type = self.check_expression(argument)?;
            actual_types.push(actual_type);
        }

        let (concrete_name, signature) = self.monomorphise_generic_function(
            &generic_name,
            generic_function,
            target.leaf_type_arguments(),
            &actual_types,
        )?;

        if signature.is_throws && !handled {
            return Err(TypeError::inference(
                FrontendMessage::UnpropagatedThrowingCall {
                    resolved_name: concrete_name,
                },
            ));
        }

        self.check_pure_call(&concrete_name, &signature)?;

        let signature = call_lifetimes::instantiate(&signature, &actual_types)?;

        for ((argument, expected_type), actual_type) in arguments
            .iter_mut()
            .zip(&signature.parameter_types)
            .zip(&actual_types)
        {
            self.unify(expected_type, actual_type)
                .map_err(|error| error.at_source(argument.span))?;
            argument.value_type = self.resolve(expected_type);
        }

        target.resolve_to(concrete_name);
        let return_type = self
            .resolve_type(&signature.return_type)
            .unwrap_or_else(|_| self.resolve(&signature.return_type));

        Ok(return_type)
    }
}
