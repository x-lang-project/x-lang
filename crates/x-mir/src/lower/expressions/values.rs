use super::*;
use crate::{MirStatement, Place, Rvalue};
use x_diagnostics::backend_messages;

mod aggregates;
mod projections;

impl MirLowerer {
    pub(super) fn lower_evaluated_expression(
        &mut self,
        expression: Expression,
    ) -> Result<Operand, MirLowerError> {
        let result_type = expression.value_type;
        let value = match expression.kind {
            ExpressionKind::Assignment { target, value } => {
                return self.lower_evaluated_assignment(*target, *value);
            }
            ExpressionKind::FunctionCall { target, arguments }
                if !x_ast::intrinsics::is_catch_call(&target.runtime_spelling())
                    || self
                        .source_functions
                        .contains(target.runtime_spelling().as_ref()) =>
            {
                return self.lower_call_with_control_flow(
                    target.runtime_spelling().into_owned(),
                    arguments,
                    result_type,
                );
            }
            kind => self.lower_evaluated_rvalue(kind, &result_type)?,
        };

        let Some(value) = value else {
            return Ok(Operand::ConstantInteger(0));
        };

        if self.current_block.is_none() {
            return Ok(Operand::ConstantInteger(0));
        }

        self.emit_evaluated_value(result_type, value)
    }

    pub(super) fn emit_evaluated_value(
        &mut self,
        value_type: Type,
        value: Rvalue,
    ) -> Result<Operand, MirLowerError> {
        let result = self.allocate_local(None, value_type);
        self.emit_statement(MirStatement::Assign(Place::Local(result), value))?;

        Ok(Operand::Move(Box::new(Place::Local(result))))
    }

    fn lower_evaluated_assignment(
        &mut self,
        target: Expression,
        value: Expression,
    ) -> Result<Operand, MirLowerError> {
        let target_type = target.value_type.clone();
        let value = self.lower_evaluated_operand(value, Some(&target_type))?;

        if self.current_block.is_none() {
            return Ok(Operand::ConstantInteger(0));
        }

        let Some(destination) = self.lower_assignment_destination(target)? else {
            return Ok(Operand::ConstantInteger(0));
        };

        self.emit_statement(MirStatement::Assign(
            destination.clone(),
            Rvalue::Use(value),
        ))?;

        Ok(Operand::Copy(Box::new(destination)))
    }

    fn lower_evaluated_rvalue(
        &mut self,
        kind: ExpressionKind,
        value_type: &Type,
    ) -> Result<Option<Rvalue>, MirLowerError> {
        let value = match kind {
            ExpressionKind::BinaryOperator {
                left,
                operator,
                right,
            } => {
                let left_type = left.value_type.clone();
                let right_type = right.value_type.clone();
                let left = self.lower_evaluated_operand(*left, Some(&left_type))?;
                let right = self.lower_evaluated_operand(*right, Some(&right_type))?;

                Rvalue::BinaryOperator(operator, left, right)
            }
            ExpressionKind::UnaryOperator {
                operator,
                expression,
            } => {
                let operand_type = expression.value_type.clone();
                let operand = self.lower_evaluated_operand(*expression, Some(&operand_type))?;

                Rvalue::UnaryOperator(operator, operand)
            }
            ExpressionKind::Cast { expression, to } => {
                let operand_type = expression.value_type.clone();
                let operand = self.lower_evaluated_operand(*expression, Some(&operand_type))?;

                Rvalue::Cast {
                    expression: operand,
                    to,
                }
            }
            aggregate @ (ExpressionKind::Tuple(_)
            | ExpressionKind::Array(_)
            | ExpressionKind::ArrayRepeat { .. }
            | ExpressionKind::StructInstantiate(_)) => self.lower_evaluated_aggregate(aggregate)?,
            projection @ (ExpressionKind::FieldAccess { .. }
            | ExpressionKind::ArrayAccess { .. }
            | ExpressionKind::Dereference { .. }) => {
                return self
                    .lower_evaluated_projection(projection, value_type)
                    .map(|place| place.map(|place| Rvalue::Use(Operand::Copy(Box::new(place)))));
            }
            shape => return Err(unsupported_shape(shape)),
        };

        Ok(Some(value))
    }
}

pub(super) fn unsupported_shape(shape: ExpressionKind) -> MirLowerError {
    MirLowerError::UnsupportedExpression(
        backend_messages::control_flow_evaluation_is_not_implemented_inside_shape(shape),
    )
}
