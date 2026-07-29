use super::*;
use crate::{MirStatement, Place, Rvalue};
use x_diagnostics::backend_messages;

impl MirLowerer {
    pub(super) fn emit_evaluated_value(
        &mut self,
        value_type: Type,
        value: Rvalue,
    ) -> Result<Operand, MirLowerError> {
        let result = self.allocate_local(None, value_type);
        self.emit_statement(MirStatement::Assign(Place::Local(result), value))?;

        Ok(Operand::Move(Box::new(Place::Local(result))))
    }
}

pub(super) fn unsupported_shape(shape: ExpressionKind) -> MirLowerError {
    MirLowerError::UnsupportedExpression(
        backend_messages::control_flow_evaluation_is_not_implemented_inside_shape(shape),
    )
}
