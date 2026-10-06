use super::*;

impl Emitter<'_> {
    pub(super) fn place(&self, second_value: &MirBody, current_place: &Place) -> Result<String> {
        match current_place {
            Place::Field {
                base, field_name, ..
            } => {
                let base_type = second_value.place_type(base);
                let type_value = match &base_type {
                    Type::RawPointer { inner, .. } | Type::Reference { inner, .. } => {
                        inner.as_ref()
                    }
                    _ => &base_type,
                };

                let indirect = pointer(&base_type);
                let suffix = match type_value {
                    Type::Custom(name) => {
                        let definition = self
                            .structs
                            .get(name.as_str())
                            .ok_or_else(|| backend_messages::c_emission_unknown_record(name))?;

                        if !definition
                            .fields
                            .iter()
                            .any(|(count, _)| count == field_name)
                        {
                            return Err(backend_messages::c_emission_unknown_record_field().into());
                        }

                        format!("f{}", encoded(field_name))
                    }

                    _ => return Err(backend_messages::c_emission_field_on_nonaggregate().into()),
                };

                Ok(format!(
                    "({}){}{}",
                    self.place(second_value, base)?,
                    if indirect { "->" } else { "." },
                    suffix
                ))
            }

            _ => Err(backend_messages::c_emission_unsupported_place(current_place).into()),
        }
    }

    pub(super) fn operand(
        &self,
        second_value: &MirBody,
        o: &Operand,
        expected: Option<&Type>,
    ) -> Result<Value> {
        let mut value_type = second_value.operand_type(o);
        let code = match o {
            Operand::ConstantFloat(function) => {
                if !function.is_finite() {
                    return Err(backend_messages::c_emission_nonfinite_literal().into());
                }

                format!("{function:.17e}")
            }
            Operand::ConstantBoolean(value) => if *value { "1" } else { "0" }.into(),
            Operand::ConstantCharacter(character) => format!("{}", (*character as u32) & 255),
            Operand::ConstantString(text) => {
                let mut pool = self.strings.borrow_mut();
                let count = pool.len();
                let identifier = *pool.entry(text.clone()).or_insert(count);

                format!("(&_x_literal{identifier})")
            }
            _ => return Err(backend_messages::c_emission_type_operand().into()),
        };

        let value = Value { code, value_type };

        if let Some(type_value) = expected {
            let sign_extend = signed(&value.value_type);

            convert(value, type_value, sign_extend)
        } else {
            Ok(value)
        }
    }
}
