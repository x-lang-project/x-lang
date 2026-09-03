use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn compile_float_cast(
        &self,
        value: BasicValueEnum<'ctx>,
        to: &Type,
        source_ast_type: &Type,
        target_llvm_type: BasicTypeEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if !value.is_float_value() {
            return Err(CodeGenerationError::UnsupportedCast { to: to.clone() });
        }

        let source = value.into_float_value();
        let target = target_llvm_type.into_float_type();

        match (source_ast_type, to) {
            (Type::F32, Type::F32) | (Type::F64, Type::F64) => Ok(source.into()),
            _ => Err(CodeGenerationError::UnsupportedCast { to: to.clone() }),
        }
    }
}
