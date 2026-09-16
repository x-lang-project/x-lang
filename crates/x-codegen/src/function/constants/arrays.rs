use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn literal_array_rvalue(
        &self,
        elements: &[Operand],
        value_type: &Type,
        constants: &HashMap<LocalIdentifier, BasicValueEnum<'ctx>>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        let Type::FixedSizedArray(inner, length) = value_type else {
            return Ok(None);
        };

        if *length != elements.len() {
            return Ok(None);
        }

        let mut values = Vec::with_capacity(elements.len());

        for operand in elements {
            let Some(value) = self.literal_operand(operand, inner, constants)? else {
                return Ok(None);
            };

            values.push(self.normalize_value_for_store(
                crate::function::numeric::TypedValue {
                    value,
                    value_type: inner,
                },
                inner,
                self.codegen.resolve_llvm_type(inner)?,
                "constant_array_adjust",
            )?);
        }

        if let Some(fields) = self.codegen.soa_fields(inner) {
            let mut arrays = Vec::with_capacity(fields.len());

            for (field_index, (_, field_type)) in fields.iter().enumerate() {
                let llvm_type = self.codegen.resolve_llvm_type(field_type)?;
                let mut field_values = Vec::with_capacity(values.len());

                for value in &values {
                    let Some(value) = value
                        .into_struct_value()
                        .get_field_at_index(field_index as u32)
                    else {
                        return Ok(None);
                    };

                    field_values.push(value);
                }

                let Some(array) = literal_array(llvm_type, &field_values) else {
                    return Ok(None);
                };

                arrays.push(array);
            }

            let storage_type = self
                .codegen
                .resolve_llvm_type(value_type)?
                .into_struct_type();

            return Ok(Some(storage_type.const_named_struct(&arrays).into()));
        }

        let element_type = self.codegen.resolve_llvm_type(inner)?;

        Ok(literal_array(element_type, &values))
    }
}

fn literal_array<'ctx>(
    value_type: types::BasicTypeEnum<'ctx>,
    values: &[BasicValueEnum<'ctx>],
) -> Option<BasicValueEnum<'ctx>> {
    use inkwell::types::BasicTypeEnum;

    macro_rules! array {
        ($ty:expr, $predicate:ident, $conversion:ident) => {
            if values.iter().all(|value| value.$predicate()) {
                Some(
                    $ty.const_array(
                        &values
                            .iter()
                            .map(|value| value.$conversion())
                            .collect::<Vec<_>>(),
                    )
                    .into(),
                )
            } else {
                None
            }
        };
    }
    match value_type {
        BasicTypeEnum::IntType(value_type) => array!(value_type, is_int_value, into_int_value),
        BasicTypeEnum::FloatType(value_type) => {
            array!(value_type, is_float_value, into_float_value)
        }
        BasicTypeEnum::PointerType(value_type) => {
            array!(value_type, is_pointer_value, into_pointer_value)
        }
        BasicTypeEnum::StructType(value_type) => {
            array!(value_type, is_struct_value, into_struct_value)
        }
        BasicTypeEnum::ArrayType(value_type) => {
            array!(value_type, is_array_value, into_array_value)
        }
        BasicTypeEnum::VectorType(value_type) => {
            array!(value_type, is_vector_value, into_vector_value)
        }
        BasicTypeEnum::ScalableVectorType(_) => None,
    }
}
