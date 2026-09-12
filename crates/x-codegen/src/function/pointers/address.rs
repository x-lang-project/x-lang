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
}
