use super::*;

mod printing;

impl Emitter<'_> {
    pub(super) fn call(
        &self,
        second_value: &MirBody,
        name: &str,
        arguments: &[Operand],
    ) -> Result<Value> {
        match second_value
            .call_targets
            .get(name)
            .ok_or_else(|| backend_messages::c_emission_unresolved(name))?
        {
            ResolvedCall::Source { name, .. } => {
                let signature = self
                    .symbols
                    .get(name.as_str())
                    .ok_or_else(|| backend_messages::c_emission_external_call(name))?;

                if signature.body.is_throws {
                    return Err(backend_messages::c_emission_unchecked_throwing_call().into());
                }

                self.source_call(second_value, signature, arguments)
            }
            ResolvedCall::Builtin(_) => {
                let builtin = second_value
                    .builtin_call(name, arguments, None)
                    .ok_or_else(backend_messages::c_emission_ambiguous_builtin)?;

                self.print_call(second_value, builtin.kind, arguments)
            }
        }
    }

    pub(super) fn throwing_call(
        &self,
        second_value: &MirBody,
        name: &str,
        arguments: &[Operand],
    ) -> Result<(String, Type)> {
        let Some(ResolvedCall::Source { name, .. }) = second_value.call_targets.get(name) else {
            return Err(backend_messages::c_emission_non_source_throwing_call().into());
        };

        let signature = self.symbols.get(name.as_str()).ok_or_else(|| {
            EmitError::internal(backend_messages::c_emission_missing_throwing_callee())
        })?;

        if !signature.body.is_throws || arguments.len() != signature.parameters.len() {
            return Err(EmitError::internal(
                backend_messages::c_emission_throws_callee_contract_arity(),
            ));
        }

        let value = self.source_call(second_value, signature, arguments)?;

        Ok((value.code, value.value_type))
    }

    fn source_call(
        &self,
        body: &MirBody,
        signature: &Signature<'_>,
        arguments: &[Operand],
    ) -> Result<Value> {
        if arguments.len() != signature.parameters.len() {
            return Err(EmitError::internal(
                backend_messages::c_emission_call_arity(),
            ));
        }

        let values = arguments
            .iter()
            .zip(signature.parameters)
            .map(|(argument, parameter_type)| {
                let value = self.operand(body, argument, None)?;
                let sign_extend = signed(&value.value_type);

                convert(value, parameter_type, sign_extend).map(|value| value.code)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Value {
            code: format!("{}({})", signature.symbol, values.join(",")),
            value_type: signature.return_value.clone(),
        })
    }
}
