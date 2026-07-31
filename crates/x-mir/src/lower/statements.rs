use super::MirLowerer;
use crate::errors::MirLowerError;
use crate::{MirStatement, MirTerminator};
use x_ast::{ExpressionKind, Pattern, Statement};
use x_diagnostics::backend_messages;

mod bindings;
mod branches;
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
            Statement::Return { value } => self.lower_return_statement(value)?,
            Statement::VariableDeclaration {
                pattern,
                type_annotation,
                value,
            } => match pattern {
                Pattern::Identifier(name) => {
                    self.lower_identifier_binding(name, type_annotation, value)?
                }
                Pattern::Tuple(names) => self.lower_tuple_binding(names, value)?,
                Pattern::Wildcard => {
                    self.lower_expression_operand(value, type_annotation.as_ref())?;
                }
                Pattern::Enum { .. } => {
                    return Err(MirLowerError::UnsupportedExpression(
                        backend_messages::expected_an_irrefutable_pattern(),
                    ));
                }
            },
            Statement::Expression { expression } => {
                if matches!(expression.kind, ExpressionKind::Match { .. }) {
                    self.lower_match_statement(expression)?;

                    return Ok(());
                }

                self.lower_expression_operand(expression, None)?;
            }

            Statement::Assert { condition, .. } => {
                self.lower_assert_statement(condition)?;
            }

            Statement::Panic { .. } => {
                self.lower_panic_statement()?;
            }

            Statement::Throw { message } => {
                self.lower_throw_statement(message)?;
            }

            Statement::If {
                condition,
                then_block,
                else_block,
            } => self.lower_if_statement(*condition, then_block, else_block)?,
            Statement::WhileLoop { condition, body } => {
                self.lower_while_statement(*condition, body)?
            }
            Statement::ForRangeLoop {
                variable,
                start,
                end,
                body,
                is_parallel,
            } => {
                if is_parallel {
                    self.lower_parallel_range(variable, *start, *end, body)?;
                } else {
                    self.lower_serial_range(variable, *start, *end, body)?;
                }
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
            Statement::Become { call } => self.lower_become_statement(call)?,
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
