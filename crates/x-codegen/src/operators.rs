mod arithmetic;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::values::BasicValueEnum;
use inkwell::{FloatPredicate, IntPredicate};

use crate::errors::CodeGenerationError;
use x_ast::Operator;
use x_diagnostics::backend_messages;

#[derive(Clone, Copy)]
pub enum IntegerSignedness {
    Signed,
    Unsigned,
}

impl IntegerSignedness {
    fn is_signed(self) -> bool {
        matches!(self, Self::Signed)
    }
}

pub struct OperatorEmitter<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
}

impl<'ctx> OperatorEmitter<'_, 'ctx> {
    pub fn emit_binary_operator(
        &self,
        operator: &Operator,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        integer_signedness: IntegerSignedness,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if left.is_pointer_value() && right.is_pointer_value() {
            let predicate = match operator {
                Operator::Equal => IntPredicate::EQ,
                Operator::NotEqual => IntPredicate::NE,
                Operator::LessThan => IntPredicate::ULT,
                Operator::GreaterThan => IntPredicate::UGT,
                Operator::LessThanOrEqual => IntPredicate::ULE,
                Operator::GreaterThanOrEqual => IntPredicate::UGE,
                _ => {
                    return Err(CodeGenerationError::UnsupportedOperator {
                        operator: operator.clone(),
                    });
                }
            };

            return self
                .builder
                .build_int_compare(
                    predicate,
                    left.into_pointer_value(),
                    right.into_pointer_value(),
                    "pointer_cmp",
                )
                .map(Into::into)
                .map_err(Into::into);
        }

        let (left, right) = Self::normalize_binary_operands(left, right)?;

        match operator {
            Operator::Add => self.emit_add(left, right),
            Operator::Subtract => self.emit_sub(left, right),
            Operator::Multiply => self.emit_mul(left, right),
            Operator::Divide => self.emit_div(left, right, integer_signedness),
            Operator::Modulo => self.emit_mod(left, right, integer_signedness),
            Operator::Equal => self.emit_comparison(
                left,
                right,
                (IntPredicate::EQ, "ieq_tmp"),
                (FloatPredicate::OEQ, "feq_tmp"),
            ),
            // NaN compares unequal.
            Operator::NotEqual => self.emit_comparison(
                left,
                right,
                (IntPredicate::NE, "ine_tmp"),
                (FloatPredicate::UNE, "fne_tmp"),
            ),
            Operator::LessThan => self.emit_comparison(
                left,
                right,
                (
                    if integer_signedness.is_signed() {
                        IntPredicate::SLT
                    } else {
                        IntPredicate::ULT
                    },
                    "icmplt_tmp",
                ),
                (FloatPredicate::OLT, "fcmplt_tmp"),
            ),
            Operator::GreaterThan => self.emit_comparison(
                left,
                right,
                (
                    if integer_signedness.is_signed() {
                        IntPredicate::SGT
                    } else {
                        IntPredicate::UGT
                    },
                    "icmpgt_tmp",
                ),
                (FloatPredicate::OGT, "fcmpgt_tmp"),
            ),
            Operator::LessThanOrEqual => self.emit_comparison(
                left,
                right,
                (
                    if integer_signedness.is_signed() {
                        IntPredicate::SLE
                    } else {
                        IntPredicate::ULE
                    },
                    "icmple_tmp",
                ),
                (FloatPredicate::OLE, "fcmple_tmp"),
            ),
            Operator::GreaterThanOrEqual => self.emit_comparison(
                left,
                right,
                (
                    if integer_signedness.is_signed() {
                        IntPredicate::SGE
                    } else {
                        IntPredicate::UGE
                    },
                    "icmpge_tmp",
                ),
                (FloatPredicate::OGE, "fcmpge_tmp"),
            ),
            Operator::And => self
                .builder
                .build_and(left.into_int_value(), right.into_int_value(), "and_tmp")
                .map(|value| value.into())
                .map_err(Into::into),
            Operator::Or => self
                .builder
                .build_or(left.into_int_value(), right.into_int_value(), "or_tmp")
                .map(|value| value.into())
                .map_err(Into::into),
            Operator::BitAnd => self
                .builder
                .build_and(left.into_int_value(), right.into_int_value(), "bitand_tmp")
                .map(|value| value.into())
                .map_err(Into::into),
            Operator::BitOr => self
                .builder
                .build_or(left.into_int_value(), right.into_int_value(), "bitor_tmp")
                .map(|value| value.into())
                .map_err(Into::into),
            Operator::Xor => self
                .builder
                .build_xor(left.into_int_value(), right.into_int_value(), "xor_tmp")
                .map(|value| value.into())
                .map_err(Into::into),
            Operator::ShiftLeft => self
                .builder
                .build_left_shift(left.into_int_value(), right.into_int_value(), "shl_tmp")
                .map(|value| value.into())
                .map_err(Into::into),
            Operator::ShiftRight => self
                .builder
                .build_right_shift(
                    left.into_int_value(),
                    right.into_int_value(),
                    integer_signedness.is_signed(),
                    "shr_tmp",
                )
                .map(|value| value.into())
                .map_err(Into::into),
        }
    }

    pub(crate) fn normalize_binary_operands(
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, BasicValueEnum<'ctx>), CodeGenerationError> {
        if left.get_type() == right.get_type() {
            Ok((left, right))
        } else {
            Err(CodeGenerationError::LlvmInternalError(backend_messages::type_mismatch_invariant_violated_before_operator_emission_value_vs_value(left.get_type(), right.get_type())))
        }
    }

    fn emit_comparison(
        &self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        integer: (IntPredicate, &str),
        float: (FloatPredicate, &str),
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let value = if left.is_float_value() {
            self.builder.build_float_compare(
                float.0,
                left.into_float_value(),
                right.into_float_value(),
                float.1,
            )?
        } else {
            self.builder.build_int_compare(
                integer.0,
                left.into_int_value(),
                right.into_int_value(),
                integer.1,
            )?
        };

        Ok(value.into())
    }
}
