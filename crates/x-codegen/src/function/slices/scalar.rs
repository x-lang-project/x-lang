mod copying;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};
use inkwell::{FloatPredicate, IntPredicate};

use super::{CopyEndpoint, FunctionCompiler, SliceValue};
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_slice_sum(
        &self,
        slice: SliceValue<'ctx>,
        initial: Option<&Operand>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if !slice.element.is_int_type() && !slice.element.is_float_type() {
            return Err(CodeGenerationError::UnsupportedType(slice.element_ast));
        }

        let initial = initial
            .map(|operand| self.load_operand_value_with_type(operand, &slice.element_ast))
            .transpose()?
            .unwrap_or_else(|| slice.element.const_zero());
        let accumulator = self.alloca(slice.element, "slice_sum_acc")?;
        self.store(accumulator, initial)?;
        self.counted_loop("slice_sum", slice.len, &mut |index| {
            let pointer = self.gep(slice.element, slice.data, index, "slice_sum_elem_ptr")?;
            let value = self.load(slice.element, pointer, "slice_sum_elem")?;
            let current = self.load(slice.element, accumulator, "slice_sum_acc_load")?;
            let result = if slice.element.is_int_type() {
                self.integer_add(
                    current.into_int_value(),
                    value.into_int_value(),
                    "slice_sum_next_acc",
                )?
                .into()
            } else {
                self.b()
                    .build_float_add(
                        current.into_float_value(),
                        value.into_float_value(),
                        "slice_sum_next_acc",
                    )?
                    .into()
            };

            self.store(accumulator, result)
        })?;

        self.load(slice.element, accumulator, "slice_sum_result")
    }

    fn scalar_element_equal(
        &self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        element: &Type,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        match (left, right) {
            (BasicValueEnum::IntValue(left), BasicValueEnum::IntValue(right)) => {
                self.icmp(IntPredicate::EQ, left, right, "slice_element_eq")
            }
            (BasicValueEnum::FloatValue(left), BasicValueEnum::FloatValue(right)) => self
                .b()
                .build_float_compare(FloatPredicate::OEQ, left, right, "slice_element_eq")
                .map_err(Into::into),
            (BasicValueEnum::PointerValue(left), BasicValueEnum::PointerValue(right)) => {
                let index_type = self
                    .codegen
                    .resolve_llvm_type(&Type::USize)?
                    .into_int_type();
                let left = self.pointer_to_integer(left, index_type, "slice_left_address")?;
                let right = self.pointer_to_integer(right, index_type, "slice_right_address")?;

                self.icmp(IntPredicate::EQ, left, right, "slice_element_eq")
            }
            _ => Err(CodeGenerationError::UnsupportedType(element.clone())),
        }
    }

    pub(super) fn emit_typed_slice_contains(
        &self,
        slice: &SliceValue<'ctx>,
        needle: BasicValueEnum<'ctx>,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        let SliceValue {
            data,
            len: length,
            element,
            element_ast,
            ..
        } = slice;
        let (data, length, element) = (*data, *length, *element);

        if *element_ast == Type::I32 {
            return self.emit_i32_slice_contains(data, length, needle.into_int_value());
        }

        self.scalar_element_equal(element.const_zero(), needle, element_ast)?;
        let result = self.alloca(self.boolean_type().into(), "slice_contains_result")?;
        self.store(result, self.boolean_type().const_zero().into())?;
        self.counted_loop("slice_contains", length, &mut |index| {
            let pointer = self.gep(element, data, index, "slice_contains_elem_ptr")?;
            let value = self.load(element, pointer, "slice_contains_elem")?;
            let equal = self.scalar_element_equal(value, needle, element_ast)?;
            let current = self
                .load(self.boolean_type().into(), result, "slice_contains_current")?
                .into_int_value();
            let found = self.integer_or(current, equal, "slice_contains_found")?;

            self.store(result, found.into())
        })?;

        self.load(self.boolean_type().into(), result, "slice_contains_result")
            .map(|value| value.into_int_value())
    }

    pub(super) fn emit_typed_slice_has_duplicate(
        &self,
        data: PointerValue<'ctx>,
        length: IntValue<'ctx>,
        element: BasicTypeEnum<'ctx>,
        element_ast: &Type,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        if *element_ast == Type::I32 {
            return self.emit_i32_slice_has_duplicate(data, length);
        }

        self.scalar_element_equal(element.const_zero(), element.const_zero(), element_ast)?;
        let result = self.alloca(self.boolean_type().into(), "slice_duplicate_result")?;
        self.store(result, self.boolean_type().const_zero().into())?;
        self.counted_loop("slice_duplicate_outer", length, &mut |outer| {
            let pointer = self.gep(element, data, outer, "slice_duplicate_left_ptr")?;
            let left = self.load(element, pointer, "slice_duplicate_left")?;

            self.counted_loop("slice_duplicate_inner", length, &mut |inner| {
                let pointer = self.gep(element, data, inner, "slice_duplicate_right_ptr")?;
                let right = self.load(element, pointer, "slice_duplicate_right")?;
                let equal = self.scalar_element_equal(left, right, element_ast)?;
                let later = self.icmp(IntPredicate::UGT, inner, outer, "slice_duplicate_later")?;
                let equal = self.integer_and(equal, later, "slice_duplicate_pair")?;
                let current = self
                    .load(
                        self.boolean_type().into(),
                        result,
                        "slice_duplicate_current",
                    )?
                    .into_int_value();
                let found = self.integer_or(current, equal, "slice_duplicate_found")?;

                self.store(result, found.into())
            })
        })?;

        self.load(self.boolean_type().into(), result, "slice_duplicate_result")
            .map(|value| value.into_int_value())
    }
}
