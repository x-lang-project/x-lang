use super::MirLowerer;
use crate::errors::MirLowerError;
use crate::{LocalIdentifier, MirStatement, MirTerminator, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::backend_messages;
use x_diagnostics::mir_messages;

impl MirLowerer {
    pub(super) fn lower_panic_statement(&mut self) -> Result<(), MirLowerError> {
        self.terminate_block(MirTerminator::Abort)
    }

    pub(super) fn lower_throw_statement(
        &mut self,
        message: Expression,
    ) -> Result<(), MirLowerError> {
        let error = self.lower_buffered_expression(message, Some(&Type::String))?;

        self.terminate_block(MirTerminator::Throw { error })
    }

    pub(super) fn lower_return_statement(
        &mut self,
        value: Option<Expression>,
    ) -> Result<(), MirLowerError> {
        if value.is_none()
            && self
                .locals
                .get(&LocalIdentifier(0))
                .is_some_and(|declaration| declaration.value_type != Type::Void)
        {
            return Err(MirLowerError::UnsupportedExpression(
                mir_messages::missing_return_value(),
            ));
        }

        if let Some(expression) = value {
            if let Some(return_declaration) = self.locals.get(&LocalIdentifier(0))
                && return_declaration.value_type != Type::Void
                && matches!(expression.kind, ExpressionKind::If { .. })
            {
                let return_type = return_declaration.value_type.clone();
                self.lower_if_expression_into_place(expression, LocalIdentifier(0), return_type)?;
                self.terminate_block(MirTerminator::Return)?;

                return Ok(());
            }

            let expected = self
                .locals
                .get(&LocalIdentifier(0))
                .map(|declaration| declaration.value_type.clone());
            let source_operand = self.lower_expression_operand(expression, expected.as_ref())?;

            if self.current_block.is_none() {
                return Ok(());
            }

            if let Some(return_declaration) = self.locals.get(&LocalIdentifier(0))
                && return_declaration.value_type != Type::Void
            {
                self.emit_statement(MirStatement::Assign(
                    Place::Local(LocalIdentifier(0)),
                    Rvalue::Use(source_operand),
                ))?;
            }
        }

        self.terminate_block(MirTerminator::Return)?;

        Ok(())
    }

    pub(super) fn lower_become_statement(&mut self, call: Expression) -> Result<(), MirLowerError> {
        let ExpressionKind::FunctionCall { target, arguments } = call.kind else {
            return Err(MirLowerError::UnsupportedExpression(
                backend_messages::become_requires_a_direct_function_call(),
            ));
        };

        let mut lowered_arguments = Vec::with_capacity(arguments.len());

        for argument in arguments {
            let value_type = argument.value_type.clone();
            lowered_arguments.push(self.lower_range_snapshot(argument, value_type)?);
        }

        self.terminate_block(MirTerminator::TailCall {
            function: target.runtime_spelling().into_owned(),
            arguments: lowered_arguments,
        })?;

        Ok(())
    }
}
