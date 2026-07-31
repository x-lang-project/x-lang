use super::ExpressionLowerer;
use super::helpers::method_call_function_name;
use crate::errors::MirLowerError;
use crate::{MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, Type, UnaryOperator};
use x_diagnostics::backend_messages;
use x_diagnostics::mir_messages;

mod preparation;
pub(crate) use preparation::PreparedCall;

impl ExpressionLowerer<'_> {
    pub(super) fn lower_method_call(
        &mut self,
        object: Expression,
        method: String,
        arguments: Vec<Expression>,
        result_type: Type,
    ) -> Result<Operand, MirLowerError> {
        if nominal_receiver(&object.value_type) {
            return Err(MirLowerError::UnsupportedExpression(
                mir_messages::checked_nominal_method_required(),
            ));
        }

        let receiver_type = object.value_type.clone();
        let mut lowered_arguments = Vec::with_capacity(arguments.len() + 1);
        lowered_arguments.push(self.lower_expression_with_context(object, None)?);
        self.lower_call_arguments_into(arguments, &mut lowered_arguments)?;

        Ok(self.emit_temporary(
            result_type,
            Rvalue::Call {
                function: method_call_function_name(&receiver_type, &method),
                arguments: lowered_arguments,
            },
        ))
    }

    pub(super) fn lower_function_call(
        &mut self,
        name: String,
        arguments: Vec<Expression>,
        current_result_type: Type,
    ) -> Result<Operand, MirLowerError> {
        let is_intrinsic = !self.source_functions.contains(&name);

        if is_intrinsic && x_ast::intrinsics::is_catch_call(&name) {
            return self.lower_catch_call(arguments, current_result_type);
        }

        let call = PreparedCall::new(name, current_result_type, self.source_functions);
        let mut lowered_arguments = Vec::with_capacity(arguments.len());

        for argument in arguments {
            let operand = match call.symbolic_argument(&argument) {
                Some(operand) => operand,
                None => self.lower_expression_with_context(argument, None)?,
            };

            lowered_arguments.push(operand);
        }

        Ok(self.emit_temporary(
            call.result_type.clone(),
            call.into_rvalue(lowered_arguments),
        ))
    }

    fn lower_call_arguments_into(
        &mut self,
        arguments: Vec<Expression>,
        lowered: &mut Vec<Operand>,
    ) -> Result<(), MirLowerError> {
        for argument in arguments {
            lowered.push(self.lower_expression_with_context(argument, None)?);
        }

        Ok(())
    }

    fn lower_catch_call(
        &mut self,
        arguments: Vec<Expression>,
        result_type: Type,
    ) -> Result<Operand, MirLowerError> {
        if arguments.len() != 2 {
            return Err(MirLowerError::UnsupportedExpression(
                mir_messages::catch_argument_count(),
            ));
        }

        let mut arguments = arguments.into_iter();
        let throwing = arguments.next().ok_or_else(|| {
            MirLowerError::UnsupportedExpression(
                backend_messages::catch_is_missing_its_throwing_call(),
            )
        })?;
        let fallback_expression = arguments.next().ok_or_else(|| {
            MirLowerError::UnsupportedExpression(
                backend_messages::catch_is_missing_its_fallback_value(),
            )
        })?;
        let ExpressionKind::FunctionCall {
            target,
            arguments: call_arguments,
        } = throwing.kind
        else {
            return Err(MirLowerError::UnsupportedExpression(
                backend_messages::catch_requires_a_direct_throwing_call(),
            ));
        };

        let function = target.runtime_spelling().into_owned();

        let mut lowered_arguments = Vec::with_capacity(call_arguments.len());
        self.lower_call_arguments_into(call_arguments, &mut lowered_arguments)?;
        let fallback = self.lower_catch_fallback(fallback_expression, &result_type)?;

        Ok(self.emit_temporary(
            result_type,
            Rvalue::CatchCall {
                function,
                arguments: lowered_arguments,
                fallback,
            },
        ))
    }

    fn lower_catch_fallback(
        &mut self,
        fallback_expression: Expression,
        result_type: &Type,
    ) -> Result<Operand, MirLowerError> {
        let before_fallback = self.statements_buffer.len();
        let literal = matches!(
            fallback_expression.kind,
            ExpressionKind::Integer(_) | ExpressionKind::Float(_)
        ) || matches!(
            &fallback_expression.kind,
            ExpressionKind::UnaryOperator { operator: UnaryOperator::Negate, expression }
                if matches!(expression.kind, ExpressionKind::Integer(_) | ExpressionKind::Float(_))
        );
        // Keep negative literals as operands in checked-call fallbacks.
        let fallback = match &fallback_expression.kind {
            ExpressionKind::UnaryOperator {
                operator: UnaryOperator::Negate,
                expression,
            } => match &expression.kind {
                ExpressionKind::Integer(value) => self.lower_integer_value(
                    value.checked_neg().ok_or_else(|| {
                        MirLowerError::UnsupportedExpression(
                            backend_messages::catch_fallback_integer_overflows(),
                        )
                    })?,
                    Some(result_type),
                )?,
                ExpressionKind::Float(value) => self.lower_expression_with_context(
                    Expression::new(result_type.clone(), ExpressionKind::Float(-value)),
                    Some(result_type),
                )?,
                _ => self.lower_expression_with_context(fallback_expression, Some(result_type))?,
            },
            _ => self.lower_expression_with_context(fallback_expression, Some(result_type))?,
        };

        if self.statements_buffer.len() != before_fallback
            && !(literal
                && matches!(
                    (self.statements_buffer.get(before_fallback..), &fallback),
                    (Some([MirStatement::Assign(Place::Local(local), Rvalue::Use(
                        Operand::ConstantInteger(_) | Operand::ConstantFloat(_)
                    ))]), Operand::Move(place)) if **place == Place::Local(*local)
                ))
        {
            return Err(MirLowerError::UnsupportedExpression(
                backend_messages::catch_fallback_must_already_be_evaluated(),
            ));
        }

        Ok(fallback)
    }
}

fn nominal_receiver(value_type: &Type) -> bool {
    match value_type {
        Type::Custom(_) | Type::GenericInstance { .. } => true,
        Type::Reference { inner, .. } => nominal_receiver(inner),
        _ => false,
    }
}
