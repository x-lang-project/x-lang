use super::*;
use crate::expression_lowerer::PreparedCall;
use crate::{MirStatement, Place, Rvalue};

impl MirLowerer {
    pub(in crate::lower) fn lower_call_with_control_flow(
        &mut self,
        name: String,
        arguments: Vec<Expression>,
        current_result_type: Type,
    ) -> Result<Operand, MirLowerError> {
        let call = PreparedCall::new(name, current_result_type, &self.source_functions);
        let mut lowered = Vec::with_capacity(arguments.len());

        for argument in arguments {
            if self.current_block.is_none() {
                return Ok(Operand::ConstantInteger(0));
            }

            let operand = match call.symbolic_argument(&argument) {
                Some(operand) => operand,
                None => {
                    let value_type = argument.value_type.clone();

                    self.lower_evaluated_operand(argument, Some(&value_type))?
                }
            };

            lowered.push(operand);
        }

        if self.current_block.is_none() {
            return Ok(Operand::ConstantInteger(0));
        }

        self.emit_evaluated_value(call.result_type.clone(), call.into_rvalue(lowered))
    }

    pub(in crate::lower) fn lower_ordered_items(
        &mut self,
        items: Vec<Expression>,
    ) -> Result<Vec<Operand>, MirLowerError> {
        let mut operands = Vec::with_capacity(items.len());

        for item in items {
            if self.current_block.is_none() {
                break;
            }

            let value_type = item.value_type.clone();
            operands.push(self.lower_evaluated_operand(item, Some(&value_type))?);
        }

        Ok(operands)
    }

    /// Read a place now, before a subsequent operand can mutate its storage.
    pub(in crate::lower) fn lower_evaluated_operand(
        &mut self,
        expression: Expression,
        value_type: Option<&Type>,
    ) -> Result<Operand, MirLowerError> {
        if self.current_block.is_none() {
            return Ok(Operand::ConstantInteger(0));
        }

        let resolved_type = value_type
            .cloned()
            .unwrap_or_else(|| expression.value_type.clone());
        let operand = self.lower_expression_operand(expression, value_type)?;

        if self.current_block.is_some() && matches!(operand, Operand::Copy(_)) {
            let local = self.allocate_local(None, resolved_type);
            self.emit_statement(MirStatement::Assign(
                Place::Local(local),
                Rvalue::Use(operand),
            ))?;

            Ok(Operand::Move(Box::new(Place::Local(local))))
        } else {
            Ok(operand)
        }
    }
}
