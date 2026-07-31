use super::MirLowerer;
use crate::errors::MirLowerError;
use crate::{
    BlockIdentifier, LocalIdentifier, MirStatement, MirTerminator, Operand, Place, Rvalue,
};
use x_ast::{Expression, ExpressionKind, Operator, Statement, Type, UnaryOperator};
use x_diagnostics::backend_messages;

impl MirLowerer {
    pub(super) fn lower_assert_statement(
        &mut self,
        condition: Expression,
    ) -> Result<(), MirLowerError> {
        let success = self.make_block();
        let failure = self.make_block();
        self.lower_condition_branch(condition, success, failure)?;
        self.current_block = Some(failure);
        self.terminate_block(MirTerminator::Abort)?;
        self.current_block = Some(success);

        Ok(())
    }

    pub(in crate::lower) fn lower_if_expression_into_place(
        &mut self,
        expression: Expression,
        destination: LocalIdentifier,
        result_type: Type,
    ) -> Result<(), MirLowerError> {
        let ExpressionKind::If {
            condition,
            then_block,
            else_block,
        } = expression.kind
        else {
            return Err(MirLowerError::UnsupportedExpression(
                backend_messages::expected_if_expression(),
            ));
        };

        let else_block = else_block.ok_or_else(|| {
            MirLowerError::UnsupportedExpression(
                backend_messages::value_producing_if_requires_an_else_branch(),
            )
        })?;
        let then_basic_block = self.make_block();
        let else_basic_block = self.make_block();
        let merge_basic_block = self.make_block();

        self.lower_condition_branch(*condition, then_basic_block, else_basic_block)?;

        for (block, statements) in [
            (then_basic_block, then_block),
            (else_basic_block, else_block),
        ] {
            self.current_block = Some(block);
            self.lower_value_block_into_place(statements, destination, &result_type)?;

            if self.current_block.is_some() {
                self.terminate_block(MirTerminator::Goto(merge_basic_block))?;
            }
        }

        self.current_block = Some(merge_basic_block);

        Ok(())
    }

    pub(in crate::lower) fn lower_value_block_into_place(
        &mut self,
        mut statements: Vec<Statement>,
        destination: LocalIdentifier,
        result_type: &Type,
    ) -> Result<(), MirLowerError> {
        self.with_variable_scope(|lowerer| {
            let tail = match statements.pop() {
                Some(Statement::Expression { expression }) => Some(expression),
                Some(statement) => {
                    statements.push(statement);

                    None
                }
                None => None,
            };

            for statement in statements {
                if lowerer.current_block.is_none() {
                    break;
                }

                lowerer.lower_statement(statement)?;
            }

            if lowerer.current_block.is_some() {
                let tail = tail.ok_or_else(|| {
                    MirLowerError::UnsupportedExpression(
                        backend_messages::continuing_value_branch_requires_a_tail_expression(),
                    )
                })?;
                let operand = lowerer.lower_expression_operand(tail, Some(result_type))?;
                lowerer.emit_statement(MirStatement::Assign(
                    Place::Local(destination),
                    Rvalue::Use(operand),
                ))?;
            }

            Ok(())
        })
    }

    pub(in crate::lower) fn lower_boolean_expression_into_place(
        &mut self,
        expression: Expression,
        destination: Place,
    ) -> Result<(), MirLowerError> {
        let true_block = self.make_block();
        let false_block = self.make_block();
        let merge_block = self.make_block();

        self.lower_condition_branch(expression, true_block, false_block)?;

        for (block, place, value) in [
            (true_block, destination.clone(), true),
            (false_block, destination, false),
        ] {
            self.current_block = Some(block);
            self.emit_statement(MirStatement::Assign(
                place,
                Rvalue::Use(Operand::ConstantBoolean(value)),
            ))?;
            self.terminate_block(MirTerminator::Goto(merge_block))?;
        }
        self.current_block = Some(merge_block);

        Ok(())
    }

    pub(super) fn lower_condition_branch(
        &mut self,
        condition: Expression,
        true_target: BlockIdentifier,
        false_target: BlockIdentifier,
    ) -> Result<(), MirLowerError> {
        match condition.kind {
            ExpressionKind::BinaryOperator {
                left,
                operator: operator @ (Operator::And | Operator::Or),
                right,
            } => {
                let right_block = self.make_block();
                let (left_true, left_false) = if operator == Operator::And {
                    (right_block, false_target)
                } else {
                    (true_target, right_block)
                };

                self.lower_condition_branch(*left, left_true, left_false)?;
                self.current_block = Some(right_block);

                self.lower_condition_branch(*right, true_target, false_target)
            }
            ExpressionKind::UnaryOperator {
                operator: UnaryOperator::LogicalNot,
                expression,
            } => self.lower_condition_branch(*expression, false_target, true_target),
            _ => {
                let condition = self.lower_expression_operand(condition, Some(&Type::Boolean))?;

                self.terminate_block(MirTerminator::SwitchInteger {
                    discriminant: condition,
                    targets: vec![(1, true_target)],
                    otherwise: false_target,
                })
            }
        }
    }

    pub(super) fn lower_if_statement(
        &mut self,
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    ) -> Result<(), MirLowerError> {
        let then_basic_block = self.make_block();
        let else_basic_block = self.make_block();
        let merge_basic_block = self.make_block();

        self.lower_condition_branch(condition, then_basic_block, else_basic_block)?;

        self.current_block = Some(then_basic_block);
        for text in then_block {
            self.lower_statement(text)?;
        }

        if self.current_block.is_some() {
            self.terminate_block(MirTerminator::Goto(merge_basic_block))?;
        }

        self.current_block = Some(else_basic_block);
        if let Some(eb) = else_block {
            for text in eb {
                self.lower_statement(text)?;
            }
        }

        if self.current_block.is_some() {
            self.terminate_block(MirTerminator::Goto(merge_basic_block))?;
        }

        self.current_block = Some(merge_basic_block);

        Ok(())
    }
}
