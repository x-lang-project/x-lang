use inkwell::IntPredicate;
use inkwell::values::BasicValueEnum;

use super::super::FunctionCompiler;
use super::number_minmax_predicate;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_ast::intrinsics::{BuiltinCall, BuiltinKind, NumberOperation};
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn emit_resolved_number_intrinsic(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let callee = call.name.as_str();
        let BuiltinKind::Number(operation) = call.kind else {
            return Err(CodeGenerationError::MissingCalleeSymbol(callee.to_string()));
        };

        let i32_type = self.i32();
        let operands = self.arguments(callee, arguments);

        if matches!(
            operation,
            NumberOperation::TestSignExtension | NumberOperation::TestTruncation
        ) {
            return Ok(i32_type.const_zero().into());
        }

        if operation == NumberOperation::SaturatingSubtractUsize {
            let left = operands.get(0)?.integer(&Type::USize)?;
            let right = operands.get(1)?.integer(&Type::USize)?;
            let underflows = self.icmp(IntPredicate::ULT, left, right, "sat_sub_underflows")?;
            let diff = self.integer_sub(left, right, "sat_sub_diff")?;

            return self
                .select_integer(underflows, left.get_type().const_zero(), diff, "sat_sub")
                .map(|element| element.into());
        }

        if matches!(
            operation,
            NumberOperation::CheckedMultiplyUsize
                | NumberOperation::CheckedAddUsize
                | NumberOperation::CheckedSubtractUsize
        ) {
            return Err(CodeGenerationError::UnsupportedType(Type::USize));
        }

        if operation == NumberOperation::AbsoluteI32 {
            let value = operands.get(0)?.integer(&Type::I32)?;
            let negative = self.icmp(
                IntPredicate::SLT,
                value,
                i32_type.const_zero(),
                "abs_negative",
            )?;
            let negated = self.integer_neg(value, "abs_negated")?;

            return self
                .select_integer(negative, negated, value, "abs_i32")
                .map(|element| element.into());
        }

        if matches!(
            operation,
            NumberOperation::Minimum | NumberOperation::Maximum
        ) {
            let [left, right] = self.arguments(callee, arguments).exact()?;
            let left = self.load_operand_value(left.operand())?.into_int_value();
            let right = self.load_operand_value(right.operand())?.into_int_value();
            let pred = number_minmax_predicate(operation);
            let cmp = self.icmp(pred, left, right, "num_minmax_cmp")?;

            return self
                .select_integer(cmp, left, right, "num_minmax")
                .map(|element| element.into());
        }

        if operation == NumberOperation::Clamp {
            let [value, low, high] = self.arguments(callee, arguments).exact()?;
            let value = self.load_operand_value(value.operand())?.into_int_value();
            let low = self.load_operand_value(low.operand())?.into_int_value();
            let high = self.load_operand_value(high.operand())?.into_int_value();
            let below = self.icmp(IntPredicate::SLT, value, low, "clamp_below")?;
            let lower = self.select_integer(below, low, value, "clamp_lower")?;
            let above = self.icmp(IntPredicate::SGT, lower, high, "clamp_above")?;

            return self
                .select_integer(above, high, lower, "clamp")
                .map(|element| element.into());
        }

        if operation == NumberOperation::SignExtendU32 {
            let value = operands.get(0)?.integer(&Type::U32)?;
            let bits = operands.get(1)?.integer(&Type::I32)?;
            let shift = self.integer_sub(
                i32_type.const_int(u64::from(i32_type.get_bit_width()), false),
                bits,
                "signext_shift",
            )?;
            let shifted = self.shl(value, shift, "signext_left")?;

            return self
                .shr(shifted, shift, true, "signext")
                .map(|element| element.into());
        }

        if matches!(
            operation,
            NumberOperation::GetBitsU32 | NumberOperation::SetBitsU32
        ) {
            let value = operands.get(0)?.integer(&Type::U32)?;
            let start = operands.get(1)?.integer(&Type::I32)?;
            let count = operands.get(2)?.integer(&Type::I32)?;
            let replacement = if operation == NumberOperation::SetBitsU32 {
                Some(operands.get(3)?.integer(&Type::U32)?)
            } else {
                None
            };
            let prefix = if replacement.is_some() {
                "set_bits"
            } else {
                "get_bits"
            };

            let one = i32_type.const_int(1, false);
            let width_mask = self.shl(one, count, &format!("{prefix}_width"))?;
            let mask = self.integer_sub(width_mask, one, &format!("{prefix}_mask"))?;

            let Some(replacement) = replacement else {
                let shifted = self.shr(value, start, false, "get_bits_shift")?;

                return self
                    .integer_and(shifted, mask, "get_bits")
                    .map(|element| element.into());
            };

            let shifted_mask = self.shl(mask, start, "set_bits_shifted_mask")?;
            let cleared = self.integer_and(
                value,
                self.integer_not(shifted_mask, "set_bits_not_mask")?,
                "set_bits_cleared",
            )?;
            let shifted_replacement = self.shl(
                self.integer_and(replacement, mask, "set_bits_repl_masked")?,
                start,
                "set_bits_repl_shifted",
            )?;

            return self
                .integer_or(cleared, shifted_replacement, "set_bits")
                .map(|element| element.into());
        }

        if operation == NumberOperation::CountOnesU32 {
            return self.emit_resolved_bit_intrinsic(
                callee,
                arguments,
                x_ast::BitIntrinsic {
                    operation: x_ast::BitOperation::CountOnes,
                    width: x_ast::BitWidth::W32,
                },
            );
        }

        if operation == NumberOperation::ByteSwapU16 {
            return self.emit_resolved_bit_intrinsic(
                callee,
                arguments,
                x_ast::BitIntrinsic {
                    operation: x_ast::BitOperation::ByteSwap,
                    width: x_ast::BitWidth::W16,
                },
            );
        }

        Err(CodeGenerationError::MissingCalleeSymbol(callee.to_string()))
    }
}
