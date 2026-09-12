use super::*;

impl<'ctx> OperatorEmitter<'_, 'ctx> {
    pub(super) fn emit_add(
        &self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if left.is_float_value() {
            self.builder
                .build_float_add(
                    left.into_float_value(),
                    right.into_float_value(),
                    "fadd_tmp",
                )
                .map(|value| value.into())
                .map_err(Into::into)
        } else {
            self.builder
                .build_int_add(left.into_int_value(), right.into_int_value(), "iadd_tmp")
                .map(|value| value.into())
                .map_err(Into::into)
        }
    }

    pub(super) fn emit_sub(
        &self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if left.is_float_value() {
            self.builder
                .build_float_sub(
                    left.into_float_value(),
                    right.into_float_value(),
                    "fsub_tmp",
                )
                .map(|value| value.into())
                .map_err(Into::into)
        } else {
            self.builder
                .build_int_sub(left.into_int_value(), right.into_int_value(), "isub_tmp")
                .map(|value| value.into())
                .map_err(Into::into)
        }
    }

    pub(super) fn emit_mul(
        &self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if left.is_float_value() {
            self.builder
                .build_float_mul(
                    left.into_float_value(),
                    right.into_float_value(),
                    "fmul_tmp",
                )
                .map(|value| value.into())
                .map_err(Into::into)
        } else {
            self.builder
                .build_int_mul(left.into_int_value(), right.into_int_value(), "imul_tmp")
                .map(|value| value.into())
                .map_err(Into::into)
        }
    }

    pub(super) fn emit_div(
        &self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        signedness: IntegerSignedness,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if left.is_float_value() {
            self.builder
                .build_float_div(
                    left.into_float_value(),
                    right.into_float_value(),
                    "fdiv_tmp",
                )
                .map(|element| element.into())
                .map_err(Into::into)
        } else {
            let value = if signedness.is_signed() {
                self.builder.build_int_signed_div(
                    left.into_int_value(),
                    right.into_int_value(),
                    "idiv_tmp",
                )?
            } else {
                self.builder.build_int_unsigned_div(
                    left.into_int_value(),
                    right.into_int_value(),
                    "udiv_tmp",
                )?
            };

            Ok(value.into())
        }
    }

    pub(super) fn emit_mod(
        &self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        signedness: IntegerSignedness,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if left.is_float_value() {
            return self
                .builder
                .build_float_rem(
                    left.into_float_value(),
                    right.into_float_value(),
                    "frem_tmp",
                )
                .map(|value| value.into())
                .map_err(Into::into);
        }

        let value = if signedness.is_signed() {
            self.builder.build_int_signed_rem(
                left.into_int_value(),
                right.into_int_value(),
                "irem_tmp",
            )?
        } else {
            self.builder.build_int_unsigned_rem(
                left.into_int_value(),
                right.into_int_value(),
                "urem_tmp",
            )?
        };

        Ok(value.into())
    }
}
