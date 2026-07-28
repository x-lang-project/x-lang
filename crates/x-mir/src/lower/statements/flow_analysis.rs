use super::MirLowerer;
use x_ast::{Expression, ExpressionKind, Operator, Pattern, Statement, UnaryOperator};

impl MirLowerer {
    pub(in crate::lower) fn expression_needs_short_circuit(&self, expression: &Expression) -> bool {
        match &expression.kind {
            ExpressionKind::BinaryOperator {
                operator: Operator::And | Operator::Or,
                ..
            } => true,
            ExpressionKind::UnaryOperator {
                operator: UnaryOperator::LogicalNot,
                expression: inner_expression,
            } => self.expression_needs_short_circuit(inner_expression),
            _ => false,
        }
    }

    pub(in crate::lower) fn expression_requires_control_flow(
        &self,
        expression: &Expression,
    ) -> bool {
        if self.expression_needs_short_circuit(expression) {
            return true;
        }

        if expression.value_type == x_ast::Type::Never {
            return true;
        }

        match &expression.kind {
            ExpressionKind::Block { .. } => true,
            ExpressionKind::BinaryOperator { left, right, .. }
            | ExpressionKind::ArrayRepeat {
                value: left,
                count: right,
            }
            | ExpressionKind::ArrayAccess {
                array: left,
                index: right,
            }
            | ExpressionKind::Assignment {
                target: left,
                value: right,
            } => {
                self.expression_requires_control_flow(left)
                    || self.expression_requires_control_flow(right)
            }
            ExpressionKind::UnaryOperator {
                expression: inner_expression,
                ..
            }
            | ExpressionKind::Cast {
                expression: inner_expression,
                ..
            }
            | ExpressionKind::Try {
                expression: inner_expression,
            }
            | ExpressionKind::Dereference {
                expression: inner_expression,
            }
            | ExpressionKind::AddressOf {
                expression: inner_expression,
                ..
            }
            | ExpressionKind::FieldAccess {
                object: inner_expression,
                ..
            } => self.expression_requires_control_flow(inner_expression),
            ExpressionKind::FunctionCall { arguments, .. } => arguments
                .iter()
                .any(|argument| self.expression_requires_control_flow(argument)),
            ExpressionKind::MethodCall {
                object, arguments, ..
            } => {
                self.expression_requires_control_flow(object)
                    || arguments
                        .iter()
                        .any(|argument| self.expression_requires_control_flow(argument))
            }
            ExpressionKind::Tuple(items) | ExpressionKind::Array(items) => items
                .iter()
                .any(|item| self.expression_requires_control_flow(item)),
            ExpressionKind::StructInstantiate(initialise) => initialise
                .fields
                .iter()
                .any(|(_, value)| self.expression_requires_control_flow(value)),
            ExpressionKind::If {
                condition,
                then_block,
                else_block,
            } => crate::lower::constant_branch_value(condition, then_block, else_block.as_deref())
                .map(|value| self.expression_requires_control_flow(value))
                .unwrap_or(true),
            ExpressionKind::Match { scrutinee, arms } => {
                arms.first()
                    .is_some_and(|arm| !matches!(arm.pattern, Pattern::Enum { .. }))
                    || self.expression_requires_control_flow(scrutinee)
                    || arms.iter().any(|arm| {
                        arm.body
                            .iter()
                            .any(|statement| self.statement_requires_control_flow(statement))
                            || arm
                                .result
                                .as_ref()
                                .is_some_and(|result| self.expression_requires_control_flow(result))
                    })
            }
            _ => false,
        }
    }

    fn statement_requires_control_flow(&self, statement: &Statement) -> bool {
        match statement {
            Statement::VariableDeclaration { value, .. }
            | Statement::Constant { value, .. }
            | Statement::Expression { expression: value }
            | Statement::Return { value: Some(value) }
            | Statement::Throw { message: value }
            | Statement::Become { call: value } => self.expression_requires_control_flow(value),
            Statement::Assert { condition, message } => {
                self.expression_requires_control_flow(condition)
                    || message
                        .as_ref()
                        .is_some_and(|message| self.expression_requires_control_flow(message))
            }
            Statement::Block { statements } => statements
                .iter()
                .any(|statement| self.statement_requires_control_flow(statement)),
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.expression_requires_control_flow(condition)
                    || then_block
                        .iter()
                        .any(|statement| self.statement_requires_control_flow(statement))
                    || else_block.as_ref().is_some_and(|block| {
                        block
                            .iter()
                            .any(|statement| self.statement_requires_control_flow(statement))
                    })
            }
            Statement::WhileLoop { condition, body } => {
                self.expression_requires_control_flow(condition)
                    || body
                        .iter()
                        .any(|statement| self.statement_requires_control_flow(statement))
            }
            Statement::ForRangeLoop {
                start, end, body, ..
            } => {
                self.expression_requires_control_flow(start)
                    || self.expression_requires_control_flow(end)
                    || body
                        .iter()
                        .any(|statement| self.statement_requires_control_flow(statement))
            }
            Statement::ForEachLoop { iterator, body, .. } => {
                self.expression_requires_control_flow(iterator)
                    || body
                        .iter()
                        .any(|statement| self.statement_requires_control_flow(statement))
            }
            Statement::Panic { message } => message
                .as_ref()
                .is_some_and(|message| self.expression_requires_control_flow(message)),
            _ => false,
        }
    }
}
