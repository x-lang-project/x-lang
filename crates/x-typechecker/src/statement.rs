mod loops;

mod bindings;
mod functions;
mod transfers;

use std::collections::HashMap;

use crate::purity::ImpureOperation;
use crate::{TypeChecker, errors::TypeError};
use x_ast::SourceError;
use x_ast::{Expression, ExpressionKind, Parameter, Pattern, Statement, Type};
use x_diagnostics::frontend_messages::ExecutableTypeReason;
use x_diagnostics::frontend_messages::FrontendMessage;

impl TypeChecker {
    pub fn check_statement(&mut self, statement: &mut Statement) -> Result<(), TypeError> {
        let span = match &*statement {
            Statement::VariableDeclaration { value, .. }
            | Statement::Constant { value, .. }
            | Statement::Return { value: Some(value) }
            | Statement::Expression { expression: value } => value.span,
            Statement::If { condition, .. } | Statement::WhileLoop { condition, .. } => {
                condition.span
            }
            Statement::Assert { condition, .. } => condition.span,
            _ => statement
                .declaration_key()
                .and_then(|key| self.declaration_sources.get(&key).copied()),
        };

        self.check_statement_inner(statement)
            .map_err(|error| error.at_source(span))
    }

    fn check_statement_inner(
        &mut self,
        current_statement: &mut Statement,
    ) -> Result<(), TypeError> {
        match current_statement {
            Statement::VariableDeclaration {
                pattern,
                type_annotation,
                value,
            } => self.check_variable_declaration(pattern, type_annotation, value)?,

            Statement::Constant {
                name,
                type_annotation,
                value,
            } => {
                let final_type =
                    self.check_declaration_initializer(type_annotation, value, |expected| {
                        format!("{expected} for constant declaration {name}")
                    })?;

                let local_reference = self.local_reference_paths(value);
                self.symbols.add_variable(name.clone(), final_type);
                self.record_local_reference_paths(name, local_reference);
            }

            Statement::Expression { expression } => {
                self.check_expression(expression)?;
            }

            Statement::TypeAlias { value_type, .. } => {
                *value_type = self.resolve_type(value_type)?;
            }

            Statement::Assert { condition, message } => {
                let condition_type = self.check_expression(condition)?;
                self.unify(&Type::Boolean, &condition_type)?;

                if let Some(current_message) = message {
                    let current_message_type = self.check_expression(current_message)?;
                    self.unify(&Type::String, &current_message_type)?;
                }
            }

            Statement::Panic {
                message: Some(current_message),
            } => {
                self.require_impure_operation(ImpureOperation::Panic)?;
                let current_message_type = self.check_expression(current_message)?;
                self.unify(&Type::String, &current_message_type)?;
            }
            Statement::Panic { message: None } => {
                self.require_impure_operation(ImpureOperation::Panic)?;
            }
            Statement::Throw { message } => {
                if self.in_anonymous_function() {
                    return Err(TypeError::inference(FrontendMessage::ThrowOutsideFunction));
                }

                let current = self
                    .current_function_name()
                    .ok_or_else(|| TypeError::inference(FrontendMessage::ThrowOutsideFunction))?;
                let signature = self.symbols.get_function(current).ok_or_else(|| {
                    TypeError::inference(FrontendMessage::MissingThrowCaller {
                        current: current.to_string(),
                    })
                })?;

                if !signature.is_throws {
                    return Err(TypeError::inference(
                        FrontendMessage::NonThrowingThrowCaller {
                            current: current.to_string(),
                        },
                    ));
                }

                let message_type = self.check_expression(message)?;
                self.unify(&Type::String, &message_type)?;
            }

            Statement::Comment(_) => {}

            Statement::Return { value } => self.check_return_statement(value.as_mut())?,

            Statement::Become { call } => self.check_become_statement(call)?,

            Statement::Block { statements } => {
                self.check_scoped_statements(statements)?;
            }

            Statement::Function {
                name,
                parameters,
                body,
                generic_parameters,
                return_type,
                is_pure,
                ..
            } => {
                if generic_parameters
                    .iter()
                    .any(|parameter| !matches!(parameter, x_ast::GenericParameter::Lifetime { .. }))
                    || parameters
                        .iter()
                        .any(|parameter| Self::type_contains_placeholder(&parameter.value_type))
                    || Self::type_contains_placeholder(return_type)
                {
                    return Ok(());
                }

                self.check_function_body(
                    name.clone(),
                    *is_pure,
                    parameters,
                    body.as_deref_mut()
                        .map(Vec::as_mut_slice)
                        .unwrap_or_default(),
                )?;
            }
            Statement::ImplDeclaration(impl_definition) => {
                self.check_impl_statement(impl_definition)?
            }

            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let condition_type = self.check_expression(condition)?;

                self.unify(&Type::Boolean, &condition_type)?;

                self.check_scoped_statements(then_block)?;

                if let Some(else_statements) = else_block {
                    self.check_scoped_statements(else_statements)?;
                }
            }

            Statement::WhileLoop { condition, body } => {
                let condition_type = self.check_expression(condition)?;
                self.unify(&Type::Boolean, &condition_type)?;

                self.check_scoped_statements(body)?;
            }

            Statement::ForRangeLoop {
                variable,
                start,
                end,
                body,
                is_parallel,
            } => {
                if *is_parallel {
                    self.require_impure_operation(ImpureOperation::ParallelLoop)?;
                }

                let range_type = self.check_range_bounds(start, end, *is_parallel)?;

                self.with_lexical_scope(|checker| {
                    checker
                        .symbols
                        .add_variable(variable.clone(), range_type.clone());

                    body.iter_mut()
                        .try_for_each(|statement| checker.check_statement(statement))
                })?;
            }

            Statement::ForEachLoop {
                variable,
                iterator,
                body,
                is_parallel,
            } => {
                if *is_parallel {
                    self.require_impure_operation(ImpureOperation::ParallelLoop)?;
                }

                self.check_expression(iterator)?;

                self.with_lexical_scope(|checker| {
                    checker
                        .symbols
                        .add_variable(variable.clone(), Type::Unknown);

                    body.iter_mut()
                        .try_for_each(|statement| checker.check_statement(statement))
                })?;
            }

            Statement::StructDeclaration(definition) => {
                if let Some(invariant) = &mut definition.invariant {
                    if definition.generic_parameters.is_some() {
                        return Err(TypeError::ExecutableType {
                            value_type: Type::Custom(definition.name.clone()),
                            reason: ExecutableTypeReason::UninstantiatedGenericInvariant,
                        });
                    }

                    let invariant_type = self.with_lexical_scope(|checker| {
                        checker.symbols.add_variable(
                            invariant.receiver.name.clone(),
                            Type::Custom(definition.name.clone()),
                        );

                        checker.check_expression(&mut invariant.predicate)
                    })?;
                    self.unify(&Type::Boolean, &invariant_type)?;
                    invariant.predicate.value_type = Type::Boolean;
                }
            }

            _ => {}
        }

        Ok(())
    }
}
