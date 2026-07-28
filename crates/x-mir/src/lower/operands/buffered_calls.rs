use super::super::MirLowerer;
use crate::errors::MirLowerError;
use crate::{MirStatement, MirTerminator, Operand, Rvalue};
use x_diagnostics::backend_messages;

impl MirLowerer {
    pub(in crate::lower) fn flush_buffer_with_potential_call(
        &mut self,
        mut buffer: Vec<MirStatement>,
        value: Operand,
    ) -> Result<Operand, MirLowerError> {
        let result = (|| {
            while let Some(index) = checked_call_index(&buffer) {
                self.flush_checked_call(&mut buffer, index)?;
            }

            self.active_block_mut()
                .map(|block| block.statements.append(&mut buffer))
        })();
        // Restore scratch statements even when lowering fails.
        buffer.clear();
        self.expression_buffer = buffer;
        result?;

        Ok(value)
    }

    fn flush_checked_call(
        &mut self,
        buffer: &mut Vec<MirStatement>,
        index: usize,
    ) -> Result<(), MirLowerError> {
        let mut prefix = buffer.drain(..=index);
        let Some(MirStatement::Assign(destination, call)) = prefix.next_back() else {
            return Err(MirLowerError::UnsupportedExpression(
                backend_messages::internal_call_index_did_not_identify_an_assignment(),
            ));
        };

        self.active_block_mut()?.statements.extend(prefix);
        let target = self.make_block();
        let terminator = match call {
            Rvalue::TryCall {
                function,
                arguments,
            } => MirTerminator::TryCall {
                function,
                arguments,
                destination,
                target,
            },
            Rvalue::CatchCall {
                function,
                arguments,
                fallback,
            } => MirTerminator::CatchCall {
                function,
                arguments,
                fallback,
                destination,
                target,
            },
            _ => {
                return Err(MirLowerError::UnsupportedExpression(
                    backend_messages::internal_call_index_did_not_identify_a_try_or_catch_call(),
                ));
            }
        };

        self.terminate_block(terminator)?;
        self.current_block = Some(target);

        Ok(())
    }
}

fn checked_call_index(buffer: &[MirStatement]) -> Option<usize> {
    buffer.iter().position(|statement| {
        matches!(
            statement,
            MirStatement::Assign(_, Rvalue::TryCall { .. } | Rvalue::CatchCall { .. })
        )
    })
}
