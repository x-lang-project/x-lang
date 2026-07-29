mod buffered_calls;

use super::MirLowerer;
use crate::errors::MirLowerError;

use crate::{LocalIdentifier, MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::backend_messages;

impl MirLowerer {
    pub(super) fn ensure_operand_local(
        &mut self,
        operand: Operand,
        value_type: Type,
    ) -> Result<LocalIdentifier, MirLowerError> {
        let operand = match operand {
            Operand::Copy(place) | Operand::Move(place) => match *place {
                Place::Local(identifier) => return Ok(identifier),
                place => Operand::Copy(Box::new(place)),
            },
            other => other,
        };

        let local = self.allocate_local(None, value_type);
        self.emit_statement(MirStatement::Assign(
            Place::Local(local),
            Rvalue::Use(operand),
        ))?;

        Ok(local)
    }
}
