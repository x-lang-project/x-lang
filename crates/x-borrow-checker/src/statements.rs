mod declarations;

use std::collections::HashMap;
use std::mem;

use super::{
    BorrowCheckError, BorrowChecker, FunctionLifetimeContext, LoopExit, LoopExitKind, Region,
    VariableInfo, declared_reference_lifetime, has_non_lifetime_generic_parameter,
    parameter_origins,
};
use crate::ReferenceProjection;
use x_ast::SourceError;
use x_ast::{ExpressionKind, Pattern, Statement, Type};
use x_diagnostics::borrow_messages;

impl BorrowChecker {
    pub(super) fn check_statements(
        &mut self,
        statements: &[Statement],
    ) -> Result<(), BorrowCheckError> {
        for statement in statements {
            if self.flow.path_terminated {
                break;
            }

            self.check_statement(statement)?;
        }

        Ok(())
    }

    pub(super) fn check_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<(), BorrowCheckError> {
        let span = match statement {
            Statement::VariableDeclaration { value, .. }
            | Statement::Constant { value, .. }
            | Statement::Return { value: Some(value) }
            | Statement::Expression { expression: value } => value.span,
            _ => None,
        };

        self.check_statement_inner(statement)
            .map_err(|error| error.at_source(span))
    }

    fn check_statement_inner(&mut self, statement: &Statement) -> Result<(), BorrowCheckError> {
        match statement {
            Statement::StructDeclaration(_) => Ok(()),
            Statement::Constant { value, .. } => self.check_expression(value),
            Statement::Function {
                parameters,
                return_type,
                body,
                generic_parameters,
                ..
            } => self.check_function_statement(
                parameters,
                return_type,
                body.as_deref().map(Vec::as_slice),
                generic_parameters,
            ),
            Statement::Block { statements } => {
                self.with_scope(|this| this.check_statements(statements))
            }
            Statement::VariableDeclaration {
                pattern,
                type_annotation,
                value,
            } => self.check_variable_statement(pattern, type_annotation, value),
            Statement::Return { value } => self.check_return_statement(value.as_ref()),
            Statement::If {
                condition,
                then_block,
                else_block,
            } => self
                .check_if(condition, then_block, else_block.as_deref(), false)
                .map(|_| ()),
            Statement::WhileLoop { condition, body } => {
                self.check_loop_body(body, crate::control_flow::LoopHeader::Condition(condition))
            }
            Statement::ForRangeLoop {
                variable,
                start,
                end,
                body,
                ..
            } => {
                self.check_expression(start)?;
                self.check_expression(end)?;

                self.check_loop_body(
                    body,
                    crate::control_flow::LoopHeader::Binding {
                        name: variable,
                        value_type: start.value_type.clone(),
                    },
                )
            }
            Statement::ForEachLoop {
                variable,
                iterator,
                body,
                ..
            } => {
                self.check_expression(iterator)?;

                self.check_loop_body(
                    body,
                    crate::control_flow::LoopHeader::Binding {
                        name: variable,
                        value_type: Type::Unknown,
                    },
                )
            }
            Statement::Expression { expression } => self.check_expression(expression),
            Statement::Become { call: expression } => {
                let ExpressionKind::FunctionCall {
                    target: name,
                    arguments,
                } = &expression.kind
                else {
                    return Err(BorrowCheckError::NonLValueReference(
                        expression.kind.clone(),
                    ));
                };

                self.check_tail_call(&name.symbol_name(), arguments)?;
                self.flow.path_terminated = true;

                Ok(())
            }
            Statement::Throw {
                message: expression,
            } => {
                self.check_expression(expression)?;
                self.flow.path_terminated = true;

                Ok(())
            }
            Statement::Assert { condition, message } => {
                self.check_expression(condition)?;
                if let Some(message) = message {
                    // The message executes only on the terminating failure path.
                    self.fork().check_expression(message)?;
                }

                Ok(())
            }
            Statement::Panic { message } => {
                if let Some(message) = message {
                    self.check_expression(message)?;
                }
                self.flow.path_terminated = true;

                Ok(())
            }
            Statement::ImplDeclaration(definition) => self.check_statements(&definition.methods),
            Statement::TraitDeclaration(definition) => {
                for method in definition.methods.values().flatten() {
                    self.check_statement(method)?;
                }

                Ok(())
            }
            Statement::Import { .. }
            | Statement::TypeAlias { .. }
            | Statement::ExternFunctionDeclaration { .. }
            | Statement::Comment(_) => Ok(()),
            Statement::Break | Statement::Continue => {
                self.flow.loop_exits.push(LoopExit {
                    kind: if matches!(statement, Statement::Break) {
                        LoopExitKind::Break
                    } else {
                        LoopExitKind::Continue
                    },
                    scopes: self.flow.scopes.clone(),
                });
                self.flow.path_terminated = true;

                Ok(())
            }
        }
    }
}
