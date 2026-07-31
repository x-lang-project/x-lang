use super::MirLowerer;
use crate::errors::MirLowerError;
use crate::lower::LoopTargets;
use crate::{LocalIdentifier, MirStatement, MirTerminator, Operand, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, Operator, Statement, Type};
use x_diagnostics::backend_messages;

impl MirLowerer {
    pub(super) fn lower_parallel_range(
        &mut self,
        variable: String,
        start: Expression,
        end: Expression,
        body: Vec<Statement>,
    ) -> Result<(), MirLowerError> {
        let body = body
            .into_iter()
            .filter(|statement| !matches!(statement, Statement::Comment(_)))
            .collect::<Vec<_>>();
        let [
            Statement::Expression {
                expression:
                    Expression {
                        kind: ExpressionKind::FunctionCall { target, arguments },
                        ..
                    },
            },
        ] = body.as_slice()
        else {
            return Err(MirLowerError::UnsupportedExpression(
                backend_messages::parallel_range_body_must_be_one_direct_worker_call(),
            ));
        };

        if !matches!(arguments.first(), Some(Expression { kind: ExpressionKind::Identifier(name), .. }) if name == &variable)
        {
            return Err(MirLowerError::UnsupportedExpression(
                backend_messages::parallel_worker_must_receive_the_loop_variable_first(),
            ));
        }

        if start.value_type != Type::I32 || end.value_type != Type::I32 {
            return Err(MirLowerError::UnsupportedExpression(
                backend_messages::parallel_range_requires_i32_bounds(),
            ));
        }

        let start = self.lower_range_snapshot(start, Type::I32)?;
        let end = self.lower_range_snapshot(end, Type::I32)?;
        let captures = arguments
            .iter()
            .skip(1)
            .map(|argument| {
                self.lower_range_snapshot(argument.clone(), argument.value_type.clone())
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.emit_statement(MirStatement::ParallelFor {
            start,
            end,
            worker: target.symbol_name().into_owned(),
            captures,
        })?;

        Ok(())
    }

    pub(super) fn lower_range_snapshot(
        &mut self,
        expression: Expression,
        value_type: Type,
    ) -> Result<Operand, MirLowerError> {
        let operand = self.lower_expression_operand(expression, Some(&value_type))?;
        let local = self.allocate_local(None, value_type);
        self.emit_statement(MirStatement::Assign(
            Place::Local(local),
            Rvalue::Use(operand),
        ))?;

        Ok(Operand::Copy(Box::new(Place::Local(local))))
    }

    pub(super) fn lower_serial_range(
        &mut self,
        variable: String,
        start: Expression,
        end: Expression,
        body: Vec<Statement>,
    ) -> Result<(), MirLowerError> {
        if !start.value_type.is_integer() || start.value_type != end.value_type {
            return Err(MirLowerError::UnsupportedExpression(
                backend_messages::serial_range_requires_integer_bounds(),
            ));
        }

        let loop_type = start.value_type.clone();

        let loop_local = self.allocate_local(Some(variable.clone()), loop_type.clone());
        let start_operator = self.lower_expression_operand(start, Some(&loop_type))?;

        self.emit_statement(MirStatement::StorageLive(loop_local))?;
        self.emit_statement(MirStatement::Assign(
            Place::Local(loop_local),
            Rvalue::Use(start_operator),
        ))?;
        let end_operator = self.lower_range_snapshot(end, loop_type.clone())?;
        let condition_basic_block = self.make_block();
        let body_basic_block = self.make_block();
        let increment_basic_block = self.make_block();
        let exit_basic_block = self.make_block();

        self.terminate_block(MirTerminator::Goto(condition_basic_block))?;

        self.current_block = Some(condition_basic_block);
        let predicate = self.allocate_local(None, Type::Boolean);
        self.emit_statement_to(
            condition_basic_block,
            MirStatement::Assign(
                Place::Local(predicate),
                Rvalue::BinaryOperator(
                    Operator::LessThan,
                    Operand::Copy(Box::new(Place::Local(loop_local))),
                    end_operator,
                ),
            ),
        )?;
        self.terminate_block(MirTerminator::SwitchInteger {
            discriminant: Operand::Copy(Box::new(Place::Local(predicate))),
            targets: vec![(1, body_basic_block)],
            otherwise: exit_basic_block,
        })?;

        self.current_block = Some(body_basic_block);
        self.lower_loop_body(
            body,
            LoopTargets {
                break_target: exit_basic_block,
                continue_target: increment_basic_block,
            },
            Some((variable, loop_local)),
        )?;
        if self.current_block.is_some() {
            self.terminate_block(MirTerminator::Goto(increment_basic_block))?;
        }

        self.current_block = Some(increment_basic_block);
        let next = self.allocate_local(None, loop_type);
        self.emit_statement(MirStatement::Assign(
            Place::Local(next),
            Rvalue::BinaryOperator(
                Operator::Add,
                Operand::Copy(Box::new(Place::Local(loop_local))),
                Operand::ConstantInteger(1),
            ),
        ))?;
        self.emit_statement(MirStatement::Assign(
            Place::Local(loop_local),
            Rvalue::Use(Operand::Copy(Box::new(Place::Local(next)))),
        ))?;
        self.terminate_block(MirTerminator::Goto(condition_basic_block))?;

        self.current_block = Some(exit_basic_block);

        Ok(())
    }

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

    pub(super) fn lower_while_statement(
        &mut self,
        condition: Expression,
        body: Vec<Statement>,
    ) -> Result<(), MirLowerError> {
        let condition_basic_block = self.make_block();
        let body_basic_block = self.make_block();
        let exit_basic_block = self.make_block();

        self.terminate_block(MirTerminator::Goto(condition_basic_block))?;
        self.current_block = Some(condition_basic_block);
        self.lower_condition_branch(condition, body_basic_block, exit_basic_block)?;

        self.current_block = Some(body_basic_block);
        self.lower_loop_body(
            body,
            LoopTargets {
                break_target: exit_basic_block,
                continue_target: condition_basic_block,
            },
            None,
        )?;
        if self.current_block.is_some() {
            self.terminate_block(MirTerminator::Goto(condition_basic_block))?;
        }

        self.current_block = Some(exit_basic_block);

        Ok(())
    }
}
