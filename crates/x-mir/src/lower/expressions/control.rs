use super::*;
use crate::MirTerminator;
use x_ast::{ControlIntrinsic, intrinsics};
use x_diagnostics::backend_messages;

impl MirLowerer {
    pub(super) fn control_call<'expression>(
        &self,
        expression: &'expression Expression,
    ) -> Option<(ControlIntrinsic, &'expression [Expression])> {
        let ExpressionKind::FunctionCall { target, arguments } = &expression.kind else {
            return None;
        };

        let name = target.runtime_spelling();

        if self.source_functions.contains(name.as_ref()) {
            return None;
        }

        let call = intrinsics::BuiltinCall::resolve(&name, &[], Some(&expression.value_type))?;
        let intrinsics::BuiltinKind::Intrinsic(intrinsics::Intrinsic::Control(operation)) =
            call.kind
        else {
            return None;
        };

        Some((operation, arguments))
    }

    pub(super) fn lower_control_call(
        &mut self,
        operation: ControlIntrinsic,
        arguments: &[Expression],
    ) -> Result<Operand, MirLowerError> {
        let terminator = match (operation, arguments) {
            (ControlIntrinsic::Abort, []) => MirTerminator::Abort,
            (ControlIntrinsic::Exit, [code]) => {
                let code = self.lower_evaluated_operand(code.clone(), Some(&Type::I32))?;

                if self.current_block.is_none() {
                    return Ok(Operand::ConstantInteger(0));
                }

                MirTerminator::Exit { code }
            }
            _ => {
                return Err(MirLowerError::UnsupportedExpression(
                    backend_messages::abort_must_be_a_non_returning_mir_terminator(),
                ));
            }
        };

        self.terminate_block(terminator)?;

        Ok(Operand::ConstantInteger(0))
    }
}
