use inkwell::IntPredicate;
use inkwell::types::{BasicTypeEnum, FloatType, IntType};
use inkwell::values::{BasicValue, BasicValueEnum, FloatValue, IntValue, PointerValue};

use super::super::FunctionCompiler;
use crate::errors::CodeGenerationError;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn icmp(
        &self,
        pred: IntPredicate,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_int_compare(pred, left, right, name)
            .map_err(Into::into)
    }

    pub(in crate::function) fn select_integer(
        &self,
        condition: IntValue<'ctx>,
        then_value: IntValue<'ctx>,
        else_value: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_select(condition, then_value, else_value, name)
            .map(|value| value.into_int_value())
            .map_err(Into::into)
    }

    pub(in crate::function) fn select_pointer(
        &self,
        condition: IntValue<'ctx>,
        then_value: PointerValue<'ctx>,
        else_value: PointerValue<'ctx>,
        name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_select(condition, then_value, else_value, name)
            .map(|value| value.into_pointer_value())
            .map_err(Into::into)
    }

    pub(in crate::function) fn option_pointer(
        &self,
        condition: IntValue<'ctx>,
        pointer: PointerValue<'ctx>,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        self.select_pointer(condition, pointer, self.pointer_type().const_null(), name)
            .map(|pointer| pointer.into())
    }

    pub(in crate::function) fn integer_add(
        &self,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_int_add(left, right, name)
            .map_err(Into::into)
    }

    pub(in crate::function) fn integer_sub(
        &self,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_int_sub(left, right, name)
            .map_err(Into::into)
    }

    pub(in crate::function) fn integer_mul(
        &self,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_int_mul(left, right, name)
            .map_err(Into::into)
    }

    pub(in crate::function) fn integer_cast(
        &self,
        value: IntValue<'ctx>,
        value_type: IntType<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_int_cast(value, value_type, name)
            .map_err(Into::into)
    }

    pub(in crate::function) fn integer_cast_signed(
        &self,
        value: IntValue<'ctx>,
        value_type: IntType<'ctx>,
        signed: bool,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_int_cast_sign_flag(value, value_type, signed, name)
            .map_err(Into::into)
    }

    pub(in crate::function) fn integer_z_extend(
        &self,
        value: IntValue<'ctx>,
        value_type: IntType<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_int_z_extend(value, value_type, name)
            .map_err(Into::into)
    }

    pub(in crate::function) fn bit_cast(
        &self,
        value: BasicValueEnum<'ctx>,
        value_type: BasicTypeEnum<'ctx>,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        self.b()
            .build_bit_cast(value, value_type, name)
            .map(|value| value.as_basic_value_enum())
            .map_err(Into::into)
    }

    pub(in crate::function) fn float_ext(
        &self,
        value: FloatValue<'ctx>,
        value_type: FloatType<'ctx>,
        name: &str,
    ) -> Result<FloatValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_float_ext(value, value_type, name)
            .map_err(Into::into)
    }

    pub(in crate::function) fn float_trunc(
        &self,
        value: FloatValue<'ctx>,
        value_type: FloatType<'ctx>,
        name: &str,
    ) -> Result<FloatValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_float_trunc(value, value_type, name)
            .map_err(Into::into)
    }

    pub(in crate::function) fn float_neg(
        &self,
        value: FloatValue<'ctx>,
        name: &str,
    ) -> Result<FloatValue<'ctx>, CodeGenerationError> {
        self.b().build_float_neg(value, name).map_err(Into::into)
    }

    pub(in crate::function) fn float_to_signed_integer(
        &self,
        value: FloatValue<'ctx>,
        value_type: IntType<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_float_to_signed_int(value, value_type, name)
            .map_err(Into::into)
    }

    pub(in crate::function) fn float_to_unsigned_integer(
        &self,
        value: FloatValue<'ctx>,
        value_type: IntType<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_float_to_unsigned_int(value, value_type, name)
            .map_err(Into::into)
    }

    pub(in crate::function) fn integer_urem(
        &self,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_int_unsigned_rem(left, right, name)
            .map_err(Into::into)
    }

    pub(in crate::function) fn integer_and(
        &self,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b().build_and(left, right, name).map_err(Into::into)
    }

    pub(in crate::function) fn integer_or(
        &self,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b().build_or(left, right, name).map_err(Into::into)
    }

    pub(in crate::function) fn integer_not(
        &self,
        value: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b().build_not(value, name).map_err(Into::into)
    }

    pub(in crate::function) fn integer_neg(
        &self,
        value: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b().build_int_neg(value, name).map_err(Into::into)
    }

    pub(in crate::function) fn shl(
        &self,
        value: IntValue<'ctx>,
        shift: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_left_shift(value, shift, name)
            .map_err(Into::into)
    }

    pub(in crate::function) fn shr(
        &self,
        value: IntValue<'ctx>,
        shift: IntValue<'ctx>,
        signed: bool,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_right_shift(value, shift, signed, name)
            .map_err(Into::into)
    }
}
