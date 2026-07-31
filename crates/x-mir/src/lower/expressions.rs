mod blocks;
mod branches;
mod control;
mod evaluation;
mod values;

use super::MirLowerer;
use crate::Operand;
use crate::errors::MirLowerError;
use x_ast::{Expression, ExpressionKind, Pattern, Type};

impl MirLowerer {
    pub(super) fn lower_expression_operand(
        &mut self,
        expression: Expression,
        expected_type: Option<&Type>,
    ) -> Result<Operand, MirLowerError> {
        if let Some((operation, arguments)) = self.control_call(&expression) {
            return self.lower_control_call(operation, arguments);
        }

        if matches!(expression.kind, ExpressionKind::Block { .. }) {
            return self.lower_scoped_block(expression, expected_type);
        }

        if matches!(expression.kind, ExpressionKind::If { .. }) {
            return self.lower_if_operand(expression, expected_type);
        }

        if let ExpressionKind::Match { scrutinee, arms } = &expression.kind
            && let [arm] = arms.as_slice()
            && !matches!(arm.pattern, Pattern::Enum { .. })
        {
            let value_type = expected_type
                .cloned()
                .unwrap_or_else(|| expression.value_type.clone());

            return self.lower_match_operand(scrutinee, arm, value_type);
        }

        if self.expression_needs_short_circuit(&expression) {
            return self.lower_boolean_operand(expression);
        }

        if self.expression_requires_control_flow(&expression) {
            return self.lower_evaluated_expression(expression);
        }

        self.lower_buffered_expression(expression, expected_type)
    }
}
