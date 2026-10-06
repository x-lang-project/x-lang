use super::*;

impl Emitter<'_> {
    pub(super) fn rvalue(
        &mut self,
        second_value: &MirBody,
        result: &Rvalue,
        target: &Type,
    ) -> Result<Value> {
        match result {
            Rvalue::Array(items) => {
                let Type::FixedSizedArray(inner, count) = target else {
                    return Err(backend_messages::c_emission_dynamic_array_layout().into());
                };

                if items.len() != *count {
                    return Err(EmitError::internal(
                        backend_messages::c_emission_array_initializer_arity(),
                    ));
                }

                let values = items
                    .iter()
                    .map(|o| {
                        self.operand(second_value, o, Some(inner))
                            .map(|value| value.code)
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(Value {
                    code: format!("(({}){{{{{}}}}})", ctype(target)?, values.join(",")),
                    value_type: target.clone(),
                })
            }

            _ => Err(backend_messages::c_emission_unsupported_rvalue(result).into()),
        }
    }
}
