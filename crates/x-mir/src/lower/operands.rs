use std::mem;

mod buffered_calls;

use super::MirLowerer;
use crate::errors::MirLowerError;
use crate::expression_lowerer::ExpressionLowerer;
use crate::{LocalIdentifier, MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::backend_messages;

impl MirLowerer {
    pub(super) fn lower_assignment_destination(
        &mut self,
        expression: Expression,
    ) -> Result<Option<Place>, MirLowerError> {
        if self.current_block.is_none() {
            return Ok(None);
        }

        let value_type = expression.value_type.clone();
        let place = match expression.kind {
            ExpressionKind::FieldAccess { object, field } => {
                let Some(base) = self.lower_assignment_destination(*object)? else {
                    return Ok(None);
                };

                Place::Field {
                    base: Box::new(base),
                    field_name: field,
                    value_type,
                }
            }
            ExpressionKind::ArrayAccess { array, index } => {
                let Some(base) = self.lower_assignment_destination(*array)? else {
                    return Ok(None);
                };

                let index = self.lower_evaluated_operand(*index, Some(&Type::USize))?;

                if self.current_block.is_none() {
                    return Ok(None);
                }

                Place::Index {
                    base: Box::new(base),
                    index,
                    value_type,
                }
            }
            ExpressionKind::Dereference { expression } => {
                let pointer_type = expression.value_type.clone();
                let pointer = self.lower_evaluated_operand(*expression, Some(&pointer_type))?;

                if self.current_block.is_none() {
                    return Ok(None);
                }

                let local = self.ensure_operand_local(pointer, pointer_type)?;

                Place::Dereference {
                    base: Box::new(Place::Local(local)),
                    value_type,
                }
            }
            kind => {
                let operand =
                    self.lower_expression_operand(Expression::new(value_type, kind), None)?;

                if self.current_block.is_none() {
                    return Ok(None);
                }

                match operand {
                    Operand::Copy(place) | Operand::Move(place) => *place,
                    other => {
                        return Err(MirLowerError::InvalidMemoryPlace(
                            backend_messages::assignment_destination_is_not_addressable_other(
                                other,
                            ),
                        ));
                    }
                }
            }
        };

        Ok(Some(place))
    }

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

    pub(super) fn lower_buffered_expression(
        &mut self,
        expression: Expression,
        expected_type: Option<&Type>,
    ) -> Result<Operand, MirLowerError> {
        let mut buffer = mem::take(&mut self.expression_buffer);
        let operand = {
            let mut expression_lowerer = ExpressionLowerer {
                locals: &mut self.locals,
                variable_mappings: &self.variable_mappings,
                statements_buffer: &mut buffer,
                global_mappings: &self.global_mappings,
                source_functions: &self.source_functions,
            };

            expression_lowerer.lower_expression_with_context(expression, expected_type)
        };

        match operand {
            Ok(operand) => self.flush_buffer_with_potential_call(buffer, operand),
            Err(error) => {
                buffer.clear();
                self.expression_buffer = buffer;

                Err(error)
            }
        }
    }
}
