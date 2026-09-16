use std::slice;

use inkwell::IntPredicate;
use inkwell::values::{BasicValueEnum, IntValue};

use super::super::FunctionCompiler;
use super::{BASE_PAGE_ALIGNMENT, align_minmax_predicate};
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_ast::intrinsics::{AlignOperation, BuiltinCall, BuiltinKind};
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn emit_resolved_align_intrinsic(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let callee = call.name.as_str();
        let BuiltinKind::Align(operation) = call.kind else {
            return Err(CodeGenerationError::MissingCalleeSymbol(callee.to_string()));
        };

        let size_type = self.usize_type();
        let operands = self.arguments(callee, arguments);
        let argument = |index: usize| -> Result<IntValue<'ctx>, CodeGenerationError> {
            let value = operands.get(index)?.value(&Type::USize)?;

            if value.is_pointer_value() {
                return self.pointer_to_integer(
                    value.into_pointer_value(),
                    size_type,
                    "align_ptr_to_int",
                );
            }

            Ok(value.into_int_value())
        };

        if operation == AlignOperation::IsPowerOfTwo {
            let value = argument(0)?;
            let nonzero = self.icmp(
                IntPredicate::NE,
                value,
                size_type.const_zero(),
                "align_nonzero",
            )?;
            let minus_one =
                self.integer_sub(value, size_type.const_int(1, false), "align_pow2_minus_one")?;
            let masked = self.integer_and(value, minus_one, "align_pow2_masked")?;
            let masked_zero = self.icmp(
                IntPredicate::EQ,
                masked,
                size_type.const_zero(),
                "align_pow2_masked_zero",
            )?;

            return self
                .integer_and(nonzero, masked_zero, "align_is_pow2")
                .map(|element| element.into());
        }

        if operation == AlignOperation::IsAligned {
            let [_, align_operand] = Self::expect_arguments(callee, arguments)?;
            let value = argument(0)?;
            let align = argument(1)?;
            let valid = self
                .emit_resolved_align_intrinsic(
                    &BuiltinCall {
                        kind: BuiltinKind::Align(AlignOperation::IsPowerOfTwo),
                        ..call.clone()
                    },
                    slice::from_ref(align_operand),
                )?
                .into_int_value();
            let safe_align = self.select_integer(
                valid,
                align,
                size_type.const_int(1, false),
                "align_safe_divisor",
            )?;
            let rem = self.integer_urem(value, safe_align, "align_rem")?;
            let aligned = self.icmp(
                IntPredicate::EQ,
                rem,
                size_type.const_zero(),
                "align_rem_zero",
            )?;

            return self
                .integer_and(valid, aligned, "align_is_aligned")
                .map(|element| element.into());
        }

        if matches!(
            operation,
            AlignOperation::AlignUp | AlignOperation::PageAlignUp | AlignOperation::AlignPointer
        ) {
            let value = argument(0)?;
            let align = if operation == AlignOperation::PageAlignUp {
                size_type.const_int(BASE_PAGE_ALIGNMENT, false)
            } else {
                argument(1)?
            };

            let mask = self.integer_sub(align, size_type.const_int(1, false), "align_up_mask")?;
            let added = self.integer_add(value, mask, "align_up_added")?;
            let not_mask = self.integer_not(mask, "align_up_not_mask")?;
            let rounded = self.integer_and(added, not_mask, "align_up_result")?;

            if operation == AlignOperation::AlignPointer {
                return self
                    .integer_to_pointer(rounded, self.pointer_type(), "align_ptr_result")
                    .map(|element| element.into());
            }

            return Ok(rounded.into());
        }

        if matches!(
            operation,
            AlignOperation::AlignDown | AlignOperation::PageAlignDown
        ) {
            let value = argument(0)?;
            let align = if operation == AlignOperation::PageAlignDown {
                size_type.const_int(BASE_PAGE_ALIGNMENT, false)
            } else {
                argument(1)?
            };

            let mask = self.integer_sub(align, size_type.const_int(1, false), "align_down_mask")?;
            let not_mask = self.integer_not(mask, "align_down_not_mask")?;

            return self
                .integer_and(value, not_mask, "align_down_result")
                .map(|element| element.into());
        }

        if operation == AlignOperation::PaddingNeeded {
            let value = argument(0)?;
            let align = argument(1)?;
            let mask = self.integer_sub(align, size_type.const_int(1, false), "padding_mask")?;
            let neg_value = self.integer_neg(value, "padding_neg_value")?;

            return self
                .integer_and(neg_value, mask, "padding_needed")
                .map(|element| element.into());
        }

        if matches!(operation, AlignOperation::Maximum | AlignOperation::Minimum) {
            let left = argument(0)?;
            let right = argument(1)?;
            let pred = align_minmax_predicate(operation);
            let cmp = self.icmp(pred, left, right, "align_minmax_cmp")?;

            return self
                .select_integer(cmp, left, right, "align_minmax")
                .map(|element| element.into());
        }

        Err(CodeGenerationError::MissingCalleeSymbol(callee.to_string()))
    }
}
