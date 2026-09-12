pub(super) use predicates::*;
mod predicates;
use std::cmp::Ordering;

use inkwell::{
    IntPredicate,
    values::{BasicValueEnum, IntValue},
};

use super::FunctionCompiler;
use crate::{
    errors::CodeGenerationError,
    operators::{IntegerSignedness, OperatorEmitter},
};
use x_ast::Operator;
use x_ast::Type;
use x_diagnostics::backend_messages;

pub(super) struct TypedValue<'a, 'ctx> {
    pub value: BasicValueEnum<'ctx>,
    pub value_type: &'a Type,
}

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    fn emit_checked_integer_shift(
        &self,
        operator: &Operator,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
        signed: bool,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let width = right
            .get_type()
            .const_int(left.get_type().get_bit_width() as u64, false);
        let valid = self.icmp(IntPredicate::ULT, right, width, "shift_amount_in_range")?;
        self.trap_if_false(valid, "shift_range")?;
        let value = if matches!(operator, Operator::ShiftLeft) {
            self.shl(left, right, "shl_tmp")?
        } else {
            self.shr(left, right, signed, "shr_tmp")?
        };

        Ok(value.into())
    }

    fn emit_checked_integer_division(
        &self,
        operator: &Operator,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
        signed: bool,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let zero = right.get_type().const_zero();
        let nonzero = self.icmp(IntPredicate::NE, right, zero, "integer_divisor_nonzero")?;
        let safe = if signed {
            let width = left.get_type().get_bit_width();
            let minimum = left
                .get_type()
                .const_int(signed_integer_minimum_bits(width), false);
            let minus_one = right.get_type().const_all_ones();
            let left_is_minimum =
                self.icmp(IntPredicate::EQ, left, minimum, "integer_div_lhs_min")?;
            let right_is_minus_one = self.icmp(
                IntPredicate::EQ,
                right,
                minus_one,
                "integer_div_rhs_minus_one",
            )?;
            let overflows =
                self.integer_and(left_is_minimum, right_is_minus_one, "integer_div_overflow")?;

            self.integer_and(
                nonzero,
                self.integer_not(overflows, "integer_div_no_overflow")?,
                "integer_div_safe",
            )?
        } else {
            nonzero
        };

        self.trap_if_false(safe, "integer_division")?;

        let value = match (operator, signed) {
            (Operator::Divide, true) => self.b().build_int_signed_div(left, right, "idiv_tmp")?,
            (Operator::Divide, false) => {
                self.b().build_int_unsigned_div(left, right, "udiv_tmp")?
            }
            (Operator::Modulo, true) => self.b().build_int_signed_rem(left, right, "irem_tmp")?,
            (Operator::Modulo, false) => {
                self.b().build_int_unsigned_rem(left, right, "urem_tmp")?
            }
            _ => {
                return Err(CodeGenerationError::LlvmInternalError(
                    backend_messages::operator_operator_escaped_its_validated_backend_class(
                        operator,
                    ),
                ));
            }
        };

        Ok(value.into())
    }

    pub(super) fn emit_typed_binary(
        &self,
        operator: &Operator,
        left: TypedValue<'_, 'ctx>,
        right: TypedValue<'_, 'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let TypedValue {
            value: mut current_left,
            value_type: left_type,
        } = left;
        let TypedValue {
            value: mut current_right,
            value_type: right_type,
        } = right;

        if current_left.is_int_value() && current_right.is_int_value() {
            let left = current_left.into_int_value();
            let right = current_right.into_int_value();

            match left
                .get_type()
                .get_bit_width()
                .cmp(&right.get_type().get_bit_width())
            {
                Ordering::Less => {
                    current_left = self
                        .b()
                        .build_int_cast_sign_flag(
                            left,
                            right.get_type(),
                            is_signed_integer_type(left_type),
                            "typed_widen_lhs",
                        )?
                        .into()
                }
                Ordering::Greater => {
                    current_right = self
                        .b()
                        .build_int_cast_sign_flag(
                            right,
                            left.get_type(),
                            is_signed_integer_type(right_type),
                            "typed_widen_rhs",
                        )?
                        .into()
                }
                Ordering::Equal => {}
            }
        }

        let emitter = OperatorEmitter {
            context: self.context(),
            builder: self.b(),
        };

        if !binary_operand_kinds_are_compatible(current_left, current_right) {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::binary_operator_operator_has_non_convertible_operands_left_left_type_lowered_to(
                    operator,
                    left_type,
                    current_left.get_type(),
                    current_right.get_type(),
                ),
            ));
        }

        if current_left.is_int_value()
            && current_right.is_int_value()
            && matches!(
                operator,
                Operator::Divide | Operator::Modulo | Operator::ShiftLeft | Operator::ShiftRight
            )
        {
            let (current_left, current_right) =
                OperatorEmitter::normalize_binary_operands(current_left, current_right)?;
            let current_left = current_left.into_int_value();
            let current_right = current_right.into_int_value();
            let signed = is_signed_integer_type(left_type);

            match operator {
                Operator::Divide | Operator::Modulo => {
                    return self.emit_checked_integer_division(
                        operator,
                        current_left,
                        current_right,
                        signed,
                    );
                }
                Operator::ShiftLeft | Operator::ShiftRight => {
                    return self.emit_checked_integer_shift(
                        operator,
                        current_left,
                        current_right,
                        signed,
                    );
                }
                _ => {
                    return Err(CodeGenerationError::LlvmInternalError(
                        backend_messages::operator_operator_escaped_its_validated_backend_class(
                            operator,
                        ),
                    ));
                }
            }
        }

        if current_left.is_int_value()
            && current_right.is_int_value()
            && matches!(
                operator,
                Operator::LessThan
                    | Operator::GreaterThan
                    | Operator::LessThanOrEqual
                    | Operator::GreaterThanOrEqual
            )
        {
            let (current_left, current_right) =
                OperatorEmitter::normalize_binary_operands(current_left, current_right)?;
            let signed = is_signed_integer_type(left_type);
            let predicate = integer_comparison_predicate(operator, signed).ok_or_else(|| {
                CodeGenerationError::LlvmInternalError(
                    backend_messages::operator_operator_escaped_its_validated_backend_class(
                        operator,
                    ),
                )
            })?;

            return self
                .icmp(
                    predicate,
                    current_left.into_int_value(),
                    current_right.into_int_value(),
                    "integer_compare",
                )
                .map(Into::into);
        }

        let signedness = if is_signed_integer_type(left_type) {
            IntegerSignedness::Signed
        } else {
            IntegerSignedness::Unsigned
        };

        emitter.emit_binary_operator(operator, current_left, current_right, signedness)
    }
}
