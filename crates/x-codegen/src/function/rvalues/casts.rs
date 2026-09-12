use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn compile_pointer_cast(
        &self,
        element: BasicValueEnum<'ctx>,
        to: &Type,
        target_llvm_type: BasicTypeEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if element.is_int_value() {
            self.integer_to_pointer(
                element.into_int_value(),
                target_llvm_type.into_pointer_type(),
                "int_to_ptr",
            )
            .map(|value| value.into())
        } else if element.is_pointer_value() {
            self.bit_cast(
                element.into_pointer_value().into(),
                target_llvm_type,
                "ptr_bitcast",
            )
        } else {
            Err(CodeGenerationError::UnsupportedCast { to: to.clone() })
        }
    }

    pub(super) fn compile_float_cast(
        &self,
        element: BasicValueEnum<'ctx>,
        to: &Type,
        source_ast_type: &Type,
        target_llvm_type: BasicTypeEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if !element.is_float_value() {
            return Err(CodeGenerationError::UnsupportedCast { to: to.clone() });
        }

        let source = element.into_float_value();
        let target = target_llvm_type.into_float_type();

        match (source_ast_type, to) {
            (Type::F64, Type::F32) => self
                .float_trunc(source, target, "cast_float_trunc")
                .map(|value| value.into()),
            (Type::F32, Type::F64) => self
                .float_ext(source, target, "cast_float_ext")
                .map(|value| value.into()),
            (Type::F32, Type::F32) | (Type::F64, Type::F64) => Ok(source.into()),
            _ => Err(CodeGenerationError::UnsupportedCast { to: to.clone() }),
        }
    }

    pub(super) fn compile_integer_cast(
        &self,
        element: BasicValueEnum<'ctx>,
        to: &Type,
        source_ast_type: &Type,
        target_llvm_type: BasicTypeEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let source_is_signed = is_signed_integer_type(source_ast_type);
        let target_is_signed = is_signed_integer_type(to);
        let target = target_llvm_type.into_int_type();

        if element.is_pointer_value() {
            self.pointer_to_integer(element.into_pointer_value(), target, "cast_ptr_to_int")
                .map(|value| value.into())
        } else if element.is_float_value() && target_is_signed {
            self.float_to_signed_integer(
                element.into_float_value(),
                target,
                "cast_float_to_signed_int",
            )
            .map(|value| value.into())
        } else if element.is_float_value() {
            self.float_to_unsigned_integer(
                element.into_float_value(),
                target,
                "cast_float_to_unsigned_int",
            )
            .map(|value| value.into())
        } else if element.is_int_value() {
            self.integer_cast_signed(
                element.into_int_value(),
                target,
                source_is_signed,
                "cast_integer",
            )
            .map(|value| value.into())
        } else {
            Err(CodeGenerationError::UnsupportedCast { to: to.clone() })
        }
    }
}
