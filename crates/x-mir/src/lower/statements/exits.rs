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
}
