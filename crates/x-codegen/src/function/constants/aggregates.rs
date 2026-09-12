use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
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
