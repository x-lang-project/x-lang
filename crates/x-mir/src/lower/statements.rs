use super::MirLowerer;
use crate::errors::MirLowerError;
use crate::{MirStatement, MirTerminator};
use x_ast::{ExpressionKind, Pattern, Statement};
use x_diagnostics::backend_messages;

mod exits;
mod flow_analysis;
mod loops;

impl MirLowerer {
    pub(super) fn lower_statement(&mut self, statement: Statement) -> Result<(), MirLowerError> {
        if self.current_block.is_none() {
            if let Statement::Comment(_) = statement {
                return Ok(());
            }

            self.current_block = Some(self.make_block());
        }

        match statement {
            Statement::Panic { .. } => {
                self.lower_panic_statement()?;
            }

            Statement::ForEachLoop { .. } => {
                return Err(MirLowerError::UnsupportedExpression(
                    backend_messages::foreach_is_not_executable_use_a_range_loop(),
                ));
            }
            Statement::Break => {
                let target = self
                    .loop_targets
                    .last()
                    .ok_or_else(|| {
                        MirLowerError::UnsupportedExpression(
                            backend_messages::break_is_only_executable_inside_a_loop(),
                        )
                    })?
                    .break_target;
                self.terminate_block(MirTerminator::Goto(target))?;
            }
            Statement::Continue => {
                let target = self
                    .loop_targets
                    .last()
                    .ok_or_else(|| {
                        MirLowerError::UnsupportedExpression(
                            backend_messages::continue_is_only_executable_inside_a_loop(),
                        )
                    })?
                    .continue_target;
                self.terminate_block(MirTerminator::Goto(target))?;
            }
            Statement::Comment(text) => {
                self.emit_statement(MirStatement::Comment(text))?;
            }
            Statement::Block { statements } => {
                self.lower_statements_until_terminated(statements)?;
            }

            _ => {}
        }

        Ok(())
    }

    pub(super) fn lower_statements_until_terminated(
        &mut self,
        statements: Vec<Statement>,
    ) -> Result<(), MirLowerError> {
        for statement in statements {
            self.lower_statement(statement)?;

            if self.current_block.is_none() {
                break;
            }
        }

        Ok(())
    }
}
