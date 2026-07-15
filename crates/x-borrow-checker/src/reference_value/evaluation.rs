use std::collections::HashMap;

use super::ReferenceValue;
use crate::ReferenceProjection;
use crate::{BorrowCheckError, BorrowChecker};
use x_ast::{Expression, ExpressionKind, SourceError, Type};

impl BorrowChecker {
    pub(crate) fn check_value_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<ReferenceValue, BorrowCheckError> {
        if self.flow.path_terminated {
            return Ok(ReferenceValue::default());
        }

        let result = match &expression.kind {
            ExpressionKind::FunctionCall {
                target: name,
                arguments,
            } => self.check_call_value(&name.symbol_name(), arguments),
            ExpressionKind::MethodCall {
                object,
                target: method,
                arguments,
            } => self.check_method_value(object, &method.symbol_name(), arguments),
            ExpressionKind::Block { statements, result } => {
                self.check_block_value(statements, result.as_deref())
            }
            ExpressionKind::If {
                condition,
                then_block,
                else_block,
            } => self.check_if(condition, then_block, else_block.as_deref(), true),
            ExpressionKind::Match { scrutinee, arms } => self.check_match_value(scrutinee, arms),
            ExpressionKind::StructInstantiate(initialise) => Ok(ReferenceValue {
                fields: initialise
                    .fields
                    .iter()
                    .map(|(name, expression)| {
                        Ok((
                            ReferenceProjection::Field(name.clone()),
                            self.check_transferred_value(expression)?,
                        ))
                    })
                    .collect::<Result<_, BorrowCheckError>>()?,
                ..Default::default()
            }),
            ExpressionKind::Tuple(items) | ExpressionKind::Array(items) => Ok(ReferenceValue {
                fields: items
                    .iter()
                    .enumerate()
                    .map(|(index, expression)| {
                        Ok((
                            ReferenceProjection::Index(index as i64),
                            self.check_transferred_value(expression)?,
                        ))
                    })
                    .collect::<Result<_, BorrowCheckError>>()?,
                ..Default::default()
            }),
            ExpressionKind::ArrayRepeat { value, count } => {
                if !self.is_copy_type(&self.place_type(value))
                    && !matches!(count.kind, ExpressionKind::Integer(0 | 1))
                {
                    return Err(BorrowCheckError::UseAfterMove(
                        "repeated non-Copy array element".into(),
                    ));
                }

                let value = self.check_transferred_value(value)?;
                self.check_expression(count)?;

                Ok(ReferenceValue {
                    fields: HashMap::from([(ReferenceProjection::AnyIndex, value)]),
                    ..Default::default()
                })
            }
            ExpressionKind::FieldAccess { object, field } => {
                if self.resolve_lvalue_owner_expression(expression).is_some() {
                    self.check_expression(expression)?;

                    return self.stored_place_references(expression);
                }

                let value = self.check_value_expression(object)?;

                Ok(self
                    .projected_object_value(object, value)
                    .fields
                    .remove(&ReferenceProjection::from_field_type(
                        &self.place_type(object),
                        field,
                    ))
                    .unwrap_or_default())
            }
            ExpressionKind::ArrayAccess { array, index } => {
                if self.resolve_lvalue_owner_expression(expression).is_some() {
                    self.check_expression(expression)?;

                    return self.stored_place_references(expression);
                }

                let value = self.check_value_expression(array)?;
                let value = self.projected_object_value(array, value);
                self.check_expression(index)?;

                Ok(value.project(&[ReferenceProjection::from_index_expression(index)]))
            }
            ExpressionKind::BinaryOperator {
                left,
                operator,
                right,
            } if matches!(operator, x_ast::Operator::Add | x_ast::Operator::Subtract)
                && matches!(self.place_type(left), Type::RawPointer { .. }) =>
            {
                let value = self.check_value_expression(left)?;
                self.check_expression(right)?;

                Ok(value)
            }
            ExpressionKind::Try { expression } => self.check_value_expression(expression),
            ExpressionKind::Cast {
                expression: inner,
                to: Type::RawPointer { .. },
            } => {
                if matches!(inner.kind, ExpressionKind::AddressOf { .. }) {
                    // Raw pointers retain origin storage for frame-transfer checks.
                    self.check_expression(expression)?;

                    self.reference_value(inner)
                } else {
                    self.check_value_expression(inner)
                }
            }
            _ => {
                self.check_expression(expression)?;

                self.reference_value(expression)
            }
        };

        result.map_err(|error| error.at_source(expression.span))
    }
}
