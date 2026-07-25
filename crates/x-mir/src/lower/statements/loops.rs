use super::MirLowerer;
use crate::errors::MirLowerError;
use crate::lower::LoopTargets;
use crate::{LocalIdentifier, MirStatement, MirTerminator, Operand, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, Operator, Statement, Type};
use x_diagnostics::backend_messages;

impl MirLowerer {
    fn lower_loop_body(
        &mut self,
        body: Vec<Statement>,
        targets: LoopTargets,
        binding: Option<(String, LocalIdentifier)>,
    ) -> Result<(), MirLowerError> {
        self.with_variable_scope(|lowerer| {
            if let Some((name, local)) = binding {
                lowerer.variable_mappings.insert(name, local);
            }

            lowerer.loop_targets.push(targets);
            let result = lowerer.lower_statements_until_terminated(body);
            lowerer.loop_targets.pop();

            result
        })
    }
}
