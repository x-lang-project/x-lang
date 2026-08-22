use super::*;

impl TypeChecker {
    pub(super) fn check_registered_method(
        &mut self,
        object: &mut Expression,
        receiver: MethodReceiver,
        method: &str,
        arguments: &mut [Expression],
    ) -> Result<DeclaredMethodCall, TypeError> {
        let MethodReceiver {
            inferred: receiver_type,
            base: receiver_base,
        } = receiver;
        let method_base = method;
        let struct_name = if let Type::Custom(struct_name) = &receiver_base {
            self.resolve_struct_name(struct_name)?
        } else if let Type::GenericInstance { .. } = &receiver_base {
            format!("{receiver_base}")
        } else if matches!(receiver_base, Type::Infer(_)) {
            let mut candidates = self
                .symbols
                .struct_methods
                .iter()
                .filter(|(_, methods)| methods.contains_key(method_base))
                .map(|(struct_name, _)| struct_name);

            if let Some(candidate) = candidates.next()
                && candidates.next().is_none()
            {
                let candidate = candidate.clone();
                self.unify(&receiver_type, &Type::Custom(candidate.clone()))?;
                object.value_type = self.resolve(&receiver_type);

                candidate
            } else {
                return Err(TypeError::UnresolvedProperty {
                    field: method_base.to_owned(),
                    value_type: receiver_base,
                });
            }
        } else {
            let owner = receiver_base.to_string();

            if self
                .symbols
                .get_struct_method(&owner, method_base)
                .is_none()
            {
                return Err(TypeError::UnresolvedProperty {
                    field: method_base.to_owned(),
                    value_type: receiver_base,
                });
            }

            owner
        };

        let method_owner = self.resolve_method_owner(&struct_name, method_base)?;

        let Some(mut signature) = self
            .symbols
            .get_struct_method(&method_owner, method_base)
            .cloned()
        else {
            return Err(TypeError::UnresolvedProperty {
                field: method_base.to_owned(),
                value_type: Type::Custom(struct_name),
            });
        };

        // Method lowering has no throwing ABI.
        if signature.is_throws {
            return Err(TypeError::inference(
                FrontendMessage::UnpropagatedThrowingCall {
                    resolved_name: format!("{method_owner}::{method_base}"),
                },
            ));
        }

        let receiver_type_arguments =
            self.method_receiver_type_arguments(&receiver_base, &struct_name);

        self.specialise_method_signature(&mut signature, &method_owner, &receiver_type_arguments);
        let self_binding = HashMap::from([("Self".to_owned(), receiver_base.clone())]);
        signature.parameter_types = signature
            .parameter_types
            .iter()
            .map(|value_type| self.substitute_type(value_type, &self_binding))
            .collect();
        signature.return_type = self.substitute_type(&signature.return_type, &self_binding);

        let mut actual_types = vec![receiver_type.clone()];

        for argument in arguments.iter_mut() {
            actual_types.push(self.check_expression(argument)?);
        }
        signature = call_lifetimes::instantiate(&signature, &actual_types)?;
        let current_receiver_parameter = signature.parameter_types.first().ok_or_else(|| {
            TypeError::invalid(FrontendMessage::MissingMethodReceiver {
                struct_name: struct_name.to_string(),
                method: method.to_string(),
            })
        })?;

        if signature.parameter_types.len() != arguments.len() + 1 {
            return Err(TypeError::ArityMismatch {
                expected: signature.parameter_types.len().saturating_sub(1),
                got: arguments.len(),
                symbol_name: format!("{struct_name}::{method}"),
            });
        }

        if !matches!(
            Some(current_receiver_parameter),
            Some(Type::Reference { inner, .. }) if matches!(inner.as_ref(), Type::TypeParameter(name) if name == "Self")
        ) {
            // MIR borrows receivers; ordinary arguments keep their checked reference type.
            match (current_receiver_parameter, &receiver_type) {
                (Type::Reference { inner, .. }, Type::Custom(_) | Type::GenericInstance { .. }) => {
                    self.unify(inner, &receiver_type)?;
                }
                _ => self.unify(current_receiver_parameter, &receiver_type)?,
            }
        }

        for ((current_argument, expected_type), actual_type) in arguments
            .iter_mut()
            .zip(signature.parameter_types.iter().skip(1))
            .zip(actual_types.iter().skip(1))
        {
            self.unify(expected_type, actual_type)
                .map_err(|error| error.at_source(current_argument.span))?;
            current_argument.value_type = if is_method_generic_placeholder(expected_type) {
                self.resolve(actual_type)
            } else {
                self.resolve(expected_type)
            };
        }

        let resolved_callee = format!("{struct_name}::{method_base}");
        self.check_pure_call(&resolved_callee, &signature)?;

        Ok(DeclaredMethodCall {
            return_type: self.resolve(&signature.return_type),
            callee: resolved_callee,
            receiver_parameter: current_receiver_parameter.clone(),
        })
    }
}
