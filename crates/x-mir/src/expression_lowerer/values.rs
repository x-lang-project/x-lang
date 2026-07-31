use std::mem;

use super::ExpressionLowerer;
use crate::errors::MirLowerError;
use crate::{MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, Operator, Type, UnaryOperator};
use x_diagnostics::backend_messages;

impl ExpressionLowerer<'_> {
    pub(super) fn lower_binary_value(
        &mut self,
        left: Expression,
        operator: Operator,
        right: Expression,
        result_type: Type,
    ) -> Result<Operand, MirLowerError> {
        if matches!(left.value_type, Type::RawPointer { .. })
            && (operator == Operator::Add || operator == Operator::Subtract)
        {
            let left_type = left.value_type.clone();
            let left_operator = self.lower_expression_with_context(left, None)?;
            let mut right_operator =
                self.lower_expression_with_context(right, Some(&Type::ISize))?;

            if operator == Operator::Subtract {
                let neg_temporary = self.allocate_temporary(Type::ISize);
                self.emit_statement(MirStatement::Assign(
                    Place::Local(neg_temporary),
                    Rvalue::UnaryOperator(UnaryOperator::Negate, right_operator),
                ));
                right_operator = Operand::Move(Box::new(Place::Local(neg_temporary)));
            }

            let computed_type = if result_type == Type::Unknown {
                left_type
            } else {
                result_type.clone()
            };

            Ok(self.emit_temporary(
                computed_type,
                Rvalue::PointerOffset {
                    base: left_operator,
                    offset: right_operator,
                },
            ))
        } else {
            let left_type = left.value_type.clone();
            let right_type = right.value_type.clone();

            let left_operator = self.lower_expression_with_context(left, Some(&right_type))?;
            let right_operator = self.lower_expression_with_context(right, Some(&left_type))?;

            let computed_type = if result_type == Type::Unknown {
                left_type
            } else {
                result_type.clone()
            };

            Ok(self.emit_temporary(
                computed_type,
                Rvalue::BinaryOperator(operator, left_operator, right_operator),
            ))
        }
    }

    pub(super) fn lower_try_value(
        &mut self,
        inner_expression: Expression,
        expected_type: Option<&Type>,
    ) -> Result<Operand, MirLowerError> {
        let value = self.lower_expression_with_context(inner_expression, expected_type)?;
        let Some(MirStatement::Assign(_, rvalue)) = self.statements_buffer.last_mut() else {
            return Err(MirLowerError::UnsupportedExpression(
                backend_messages::postfix_did_not_lower_to_a_direct_function_call(),
            ));
        };

        let Rvalue::Call {
            function,
            arguments,
        } = rvalue
        else {
            return Err(MirLowerError::UnsupportedExpression(
                backend_messages::postfix_requires_a_direct_throwing_call(),
            ));
        };

        *rvalue = Rvalue::TryCall {
            function: mem::take(function),
            arguments: mem::take(arguments),
        };

        Ok(value)
    }

    pub(super) fn lower_integer_value(
        &mut self,
        index: i64,
        expected_type: Option<&Type>,
    ) -> Result<Operand, MirLowerError> {
        match expected_type {
            Some(expected) if expected.is_integer() && expected != &Type::I32 => Ok(self
                .emit_temporary(
                    expected.clone(),
                    Rvalue::Use(Operand::ConstantInteger(index)),
                )),
            _ => Ok(Operand::ConstantInteger(index)),
        }
    }
}
