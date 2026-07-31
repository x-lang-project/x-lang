use super::*;
use x_diagnostics::backend_messages;

impl MirLowerer {
    pub(super) fn lower_scoped_block(
        &mut self,
        expression: Expression,
        expected_type: Option<&Type>,
    ) -> Result<Operand, MirLowerError> {
        let ExpressionKind::Block { statements, result } = expression.kind else {
            return Err(values::unsupported_shape(expression.kind));
        };

        let block_type = expected_type.cloned().unwrap_or(expression.value_type);

        self.with_variable_scope(|lowerer| {
            for statement in statements {
                if lowerer.current_block.is_none() {
                    break;
                }

                lowerer.lower_statement(statement)?;
            }

            if lowerer.current_block.is_none() {
                return Ok(Operand::ConstantInteger(0));
            }

            match result {
                Some(result) => lowerer.lower_expression_operand(*result, expected_type),
                None if block_type == Type::Void => Ok(Operand::ConstantInteger(0)),
                None => Err(MirLowerError::UnsupportedExpression(
                    backend_messages::continuing_value_branch_requires_a_tail_expression(),
                )),
            }
        })
    }
}
