use super::MirLowerer;
use crate::errors::MirLowerError;
use crate::{LocalIdentifier, MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, MatchArm, Pattern, Statement, Type};
use x_diagnostics::backend_messages;
use x_diagnostics::mir_messages;

impl MirLowerer {
    pub(super) fn lower_irrefutable_match(
        &mut self,
        scrutinee: Expression,
        arm: &MatchArm,
        destination: Option<(LocalIdentifier, Type)>,
    ) -> Result<(), MirLowerError> {
        let value_type = scrutinee.value_type.clone();
        let payload = self.lower_evaluated_operand(scrutinee, Some(&value_type))?;

        if self.current_block.is_none() {
            return Ok(());
        }

        self.with_variable_scope(|lowerer| {
            match &arm.pattern {
                Pattern::Wildcard => {}
                Pattern::Identifier(name) => {
                    let local = lowerer.ensure_operand_local(payload, value_type)?;
                    lowerer.variable_mappings.insert(name.clone(), local);
                }
                Pattern::Tuple(bindings) => {
                    let Type::Tuple(items) = &value_type else {
                        return Err(MirLowerError::UnsupportedExpression(
                            backend_messages::tuple_pattern_requires_a_tuple(),
                        ));
                    };

                    if items.len() != bindings.len() {
                        return Err(MirLowerError::UnsupportedExpression(
                            backend_messages::tuple_pattern_arity_mismatch(),
                        ));
                    }

                    let items = items.clone();
                    let local = lowerer.ensure_operand_local(payload, value_type)?;

                    for (index, (name, value_type)) in bindings.iter().zip(items).enumerate() {
                        if name == "_" {
                            continue;
                        }

                        let binding =
                            lowerer.allocate_local(Some(name.clone()), value_type.clone());
                        lowerer.emit_statement(MirStatement::StorageLive(binding))?;
                        lowerer.emit_statement(MirStatement::Assign(
                            Place::Local(binding),
                            Rvalue::Use(Operand::Copy(Box::new(Place::Field {
                                base: Box::new(Place::Local(local)),
                                field_name: index.to_string(),
                                value_type,
                            }))),
                        ))?;
                        lowerer.variable_mappings.insert(name.clone(), binding);
                    }
                }
                Pattern::Enum { .. } => {
                    return Err(MirLowerError::UnsupportedExpression(
                        backend_messages::expected_an_irrefutable_pattern(),
                    ));
                }
            }

            if let Some((result, value_type)) = destination {
                let mut body = arm.body.clone();

                if let Some(value) = &arm.result {
                    body.push(Statement::Expression {
                        expression: value.clone(),
                    });
                }

                lowerer.lower_value_block_into_place(body, result, &value_type)
            } else {
                lowerer.lower_statements_until_terminated(arm.body.clone())?;
                if lowerer.current_block.is_some()
                    && let Some(value) = &arm.result
                {
                    lowerer.lower_expression_operand(value.clone(), None)?;
                }

                Ok(())
            }
        })
    }

    pub(super) fn lower_match_statement(
        &mut self,
        expression: Expression,
    ) -> Result<(), MirLowerError> {
        let ExpressionKind::Match { scrutinee, arms } = expression.kind else {
            return Err(MirLowerError::UnsupportedExpression(
                backend_messages::lower_match_statement_called_for_a_non_match_expression(),
            ));
        };

        if let [arm] = arms.as_slice()
            && !matches!(arm.pattern, Pattern::Enum { .. })
        {
            return self.lower_irrefutable_match(*scrutinee, arm, None);
        }

        Err(MirLowerError::UnsupportedExpression(
            mir_messages::resolved_match_layout_required(),
        ))
    }
}
