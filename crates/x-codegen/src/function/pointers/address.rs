use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn coerce_value_to_pointer(
        &self,
        value: BasicValueEnum<'ctx>,
        name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        if value.is_pointer_value() {
            return Ok(value.into_pointer_value());
        }

        if value.is_int_value() {
            return self.integer_to_pointer(value.into_int_value(), self.pointer_type(), name);
        }

        Err(CodeGenerationError::LlvmInternalError(
            backend_messages::cannot_treat_llvm_value_value_as_a_pointer(value.get_type()),
        ))
    }

    pub(in crate::function) fn emit_pointer_address_intrinsic(
        &self,
        callee: &str,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let [argument] = self.arguments(callee, arguments).exact()?;

        let value = self.load_operand_value(argument.operand())?;

        if value.is_int_value() {
            return Ok(value);
        }

        self.pointer_to_integer(value.into_pointer_value(), self.usize_type(), "ptr_addr")
            .map(|element| element.into())
    }

    pub(in crate::function) fn emit_pointer_from_address_intrinsic(
        &self,
        callee: &str,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let [argument] = self.arguments(callee, arguments).exact()?;

        let value = argument.value(&Type::USize)?;

        if value.is_pointer_value() {
            return Ok(value);
        }

        self.integer_to_pointer(value.into_int_value(), self.pointer_type(), "ptr_from_addr")
            .map(|element| element.into())
    }

    pub(in crate::function) fn emit_pointer_with_address_intrinsic(
        &self,
        callee: &str,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let [_, address] = self.arguments(callee, arguments).exact()?;

        let value = address.value(&Type::USize)?;

        if value.is_pointer_value() {
            return Ok(value);
        }

        self.integer_to_pointer(value.into_int_value(), self.pointer_type(), "ptr_with_addr")
            .map(|element| element.into())
    }
}
