use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn literal_struct(
        &self,
        name: &str,
        fields: &[(String, Operand)],
        constants: &HashMap<LocalIdentifier, BasicValueEnum<'ctx>>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        let Some(struct_type) = self.codegen.resolve_struct_type(name) else {
            return Ok(None);
        };

        let field_types = struct_type.get_field_types();

        if field_types.len() != fields.len() {
            return Ok(None);
        }

        let mut values = vec![None; fields.len()];

        for (field_name, operand) in fields {
            let index = self.struct_initialiser_field_index(name, field_name)? as usize;
            let Some(field_type) = field_types.get(index).copied() else {
                return Ok(None);
            };

            let operand_type = self.operand_ast_type(operand).unwrap_or(Type::Unknown);
            let Some(value) = self.literal_operand(operand, &operand_type, constants)? else {
                return Ok(None);
            };

            let value = self.normalize_value_for_store(
                crate::function::numeric::TypedValue {
                    value,
                    value_type: &operand_type,
                },
                &operand_type,
                field_type,
                "constant_field_adjust",
            )?;
            let Some(slot) = values.get_mut(index) else {
                return Ok(None);
            };

            if slot.replace(value).is_some() {
                return Err(CodeGenerationError::InvalidMir(
                    backend_messages::duplicate_struct_initializer_field_name_field_name(
                        name, field_name,
                    ),
                ));
            }
        }

        let Some(values) = values.into_iter().collect::<Option<Vec<_>>>() else {
            return Ok(None);
        };

        Ok(Some(struct_type.const_named_struct(&values).into()))
    }

    pub(super) fn literal_tuple(
        &self,
        elements: &[Operand],
        value_type: &Type,
        constants: &HashMap<LocalIdentifier, BasicValueEnum<'ctx>>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        let Type::Tuple(items) = value_type else {
            return Ok(None);
        };

        if items.len() != elements.len() {
            return Ok(None);
        }

        let struct_type = self
            .codegen
            .resolve_llvm_type(value_type)?
            .into_struct_type();
        let mut values = Vec::with_capacity(elements.len());

        for (operand, item_type) in elements.iter().zip(items) {
            let Some(value) = self.literal_operand(operand, item_type, constants)? else {
                return Ok(None);
            };

            values.push(self.normalize_value_for_store(
                crate::function::numeric::TypedValue {
                    value,
                    value_type: item_type,
                },
                item_type,
                self.codegen.resolve_llvm_type(item_type)?,
                "constant_tuple_adjust",
            )?);
        }

        Ok(Some(struct_type.const_named_struct(&values).into()))
    }
}
