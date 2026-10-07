use super::*;

impl Emitter<'_> {
    pub(super) fn rvalue(
        &mut self,
        second_value: &MirBody,
        result: &Rvalue,
        target: &Type,
    ) -> Result<Value> {
        match result {
            Rvalue::Use(o) => self.operand(second_value, o, Some(target)),
            Rvalue::Call {
                function,
                arguments,
            } => self.call(second_value, function, arguments),
            Rvalue::AddressOf { expression, .. } => {
                if string_element(second_value, expression) {
                    return Err(backend_messages::c_emission_address_of_string_element().into());
                }

                Ok(Value {
                    code: format!("(&({}))", self.place(second_value, expression)?),
                    value_type: target.clone(),
                })
            }
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
            Rvalue::Tuple(items) => {
                let Type::Tuple(types) = target else {
                    return Err(backend_messages::c_emission_tuple_target_layout().into());
                };

                if items.len() != types.len() {
                    return Err(EmitError::internal(
                        backend_messages::c_emission_tuple_arity(),
                    ));
                }

                let values = items
                    .iter()
                    .zip(types)
                    .map(|(o, type_value)| {
                        self.operand(second_value, o, Some(type_value))
                            .map(|value| value.code)
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(Value {
                    code: format!("(({}){{{}}})", ctype(target)?, values.join(",")),
                    value_type: target.clone(),
                })
            }
            Rvalue::StructInstantiate { name, fields } => {
                let definition = self
                    .structs
                    .get(name.as_str())
                    .ok_or_else(backend_messages::c_emission_unknown_struct_initializer)?;

                if definition.fields.len() != fields.len() {
                    return Err(backend_messages::c_emission_partial_struct_initializer().into());
                }

                if !matches!(target, Type::Custom(target_name) if target_name == name) {
                    return Err(backend_messages::c_emission_unknown_struct_initializer().into());
                }

                let mut initialized = std::collections::HashSet::new();

                for (field, _) in fields {
                    if !initialized.insert(field) {
                        return Err(
                            backend_messages::duplicate_struct_initializer_field_name_field_name(
                                name, field,
                            )
                            .into(),
                        );
                    }
                }

                let mut values = Vec::new();

                for (field, operand) in fields {
                    let (_, value_type) = definition
                        .fields
                        .iter()
                        .find(|(count, _)| count == field)
                        .ok_or_else(backend_messages::c_emission_unknown_field)?;
                    let value = self.operand(second_value, operand, None)?;
                    let sign = signed(&value.value_type);
                    let value = convert(value, value_type, sign)?;
                    values.push(format!(".f{}={}", encoded(field), value.code));
                }

                Ok(Value {
                    code: format!("(({}){{{}}})", ctype(target)?, values.join(",")),
                    value_type: target.clone(),
                })
            }
            Rvalue::Cast { expression, to } => {
                let value = self.operand(second_value, expression, None)?;

                if matches!(
                    (&value.value_type, to),
                    (Type::F32, Type::F64) | (Type::F64, Type::F32)
                ) {
                    return Ok(Value {
                        code: format!("(({})({}))", ctype(to)?, value.code),
                        value_type: to.clone(),
                    });
                }

                let sign = signed(&second_value.operand_type(expression));

                convert(value, to, sign)
            }
            Rvalue::SizeOf(type_value) => {
                let count = if let Some(writer) = width(type_value) {
                    u64::from(writer.max(8)) / 8
                } else {
                    match type_value {
                        Type::F32 => 4,
                        Type::F64
                        | Type::String
                        | Type::RawPointer { .. }
                        | Type::Reference { .. } => 8,
                        Type::Custom(_) | Type::Tuple(_) | Type::FixedSizedArray(_, _) => {
                            return Ok(Value {
                                code: format!("sizeof({})", ctype(type_value)?),
                                value_type: Type::USize,
                            });
                        }
                        _ => return Err(backend_messages::c_emission_sizeof_aggregate().into()),
                    }
                };

                Ok(Value {
                    code: format!("{count}ULL"),
                    value_type: Type::USize,
                })
            }
            Rvalue::UnaryOperator(operator, o) => {
                let value = self.operand(second_value, o, None)?;
                let code = match operator {
                    UnaryOperator::Negate if width(&value.value_type).is_some() => bits(
                        &format!("0ULL-(unsigned long long)({})", value.code),
                        width(&value.value_type).ok_or_else(|| {
                            EmitError::internal(
                                backend_messages::c_emission_numeric_value_width_is_missing(),
                            )
                        })?,
                    ),
                    UnaryOperator::Negate => format!("(-({}))", value.code),
                    UnaryOperator::BitwiseNot => bits(
                        &format!("~({})", value.code),
                        width(&value.value_type)
                            .ok_or_else(backend_messages::c_emission_float_bitwise)?,
                    ),
                    UnaryOperator::LogicalNot => {
                        return Ok(Value {
                            code: format!("!({})", value.code),
                            value_type: Type::Boolean,
                        });
                    }
                    _ => return Err(backend_messages::c_emission_unary_effect().into()),
                };

                Ok(Value {
                    code,
                    value_type: value.value_type,
                })
            }
            Rvalue::BinaryOperator(operator, left_value, result) => {
                self.binary(second_value, (operator, left_value, result))
            }
            _ => Err(backend_messages::c_emission_unsupported_rvalue(result).into()),
        }
    }
}
