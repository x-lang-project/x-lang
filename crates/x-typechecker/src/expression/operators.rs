use super::*;
use x_diagnostics::frontend_messages::FrontendMessage;

impl TypeChecker {
    pub(super) fn check_binary(
        &mut self,
        left: &mut Expression,
        operator: &Operator,
        right: &mut Expression,
    ) -> Result<Type, TypeError> {
        let left_literal = unresolved_integer_literal(left);
        let right_literal = unresolved_integer_literal(right);
        let left_type = self.check_expression(left)?;
        let right_type = self.check_expression(right)?;

        let mut resolved_left = self.resolve(&left_type);
        let mut resolved_right = self.resolve(&right_type);

        if left_literal && !right_literal && resolved_right.is_integer() {
            resolved_left = resolved_right.clone();
        } else if right_literal && !left_literal && resolved_left.is_integer() {
            resolved_right = resolved_left.clone();
        }

        let operator_type = match operator {
            Operator::Add | Operator::Subtract
                if matches!(resolved_left, Type::RawPointer { .. }) =>
            {
                if !self.is_integer_primitive(&resolved_right) {
                    return Err(crate::type_mismatch!(
                        "integer pointer offset",
                        format!("{:?}", resolved_right)
                    ));
                }

                resolved_left.clone()
            }

            Operator::Equal
            | Operator::NotEqual
            | Operator::LessThan
            | Operator::GreaterThan
            | Operator::LessThanOrEqual
            | Operator::GreaterThanOrEqual => {
                self.unify(&resolved_left, &resolved_right)?;

                Type::Boolean
            }

            _ => {
                self.unify(&resolved_left, &resolved_right)?;

                self.resolve(&resolved_left)
            }
        };

        left.value_type = self.resolve(&resolved_left);
        right.value_type = self.resolve(&resolved_right);

        Ok(operator_type)
    }

    pub(super) fn check_cast(
        &mut self,
        inner: &mut Expression,
        to: &mut Type,
    ) -> Result<Type, TypeError> {
        let inner_type = self.check_expression(inner)?;
        let target_type = self.resolve_type(to)?;

        let resolved_inner = cast_literal_source_type(inner, self.resolve(&inner_type));
        let resolved_target = self.resolve(&target_type);

        match (&resolved_inner, &resolved_target) {
            (Type::F32 | Type::F64, Type::F32 | Type::F64) => {}
            (inner, Type::RawPointer { .. }) if self.is_integer_primitive(inner) => {}
            (
                Type::RawPointer { .. } | Type::Reference { .. },
                Type::RawPointer { .. } | Type::Reference { .. },
            ) => {}
            (Type::RawPointer { .. }, target) | (Type::Reference { .. }, target)
                if self.is_integer_primitive(target) => {}
            _ => {
                self.unify(&resolved_target, &resolved_inner)?;
            }
        }

        inner.value_type = resolved_inner;

        Ok(resolved_target)
    }

    pub(super) fn check_unary(
        &mut self,
        operator: &UnaryOperator,
        inner: &mut Expression,
    ) -> Result<Type, TypeError> {
        let inner_type = self.check_expression(inner)?;
        inner.value_type = inner_type.clone();

        let base_type = Self::unwrap_type_references(inner_type);

        match operator {
            UnaryOperator::Negate => {
                if matches!(
                    base_type,
                    Type::I8
                        | Type::I16
                        | Type::I32
                        | Type::I64
                        | Type::ISize
                        | Type::F32
                        | Type::F64
                ) {
                    Ok(base_type)
                } else {
                    Err(crate::type_mismatch!(
                        "signed numeric type",
                        format!("{:?}", base_type)
                    ))
                }
            }
            UnaryOperator::LogicalNot => {
                if base_type == Type::Boolean {
                    Ok(Type::Boolean)
                } else if self.is_integer_primitive(&base_type) {
                    Ok(base_type)
                } else {
                    Err(crate::type_mismatch!(
                        "bool or integer type",
                        format!("{:?}", base_type)
                    ))
                }
            }
            UnaryOperator::PreIncrement
            | UnaryOperator::PreDecrement
            | UnaryOperator::PostIncrement
            | UnaryOperator::PostDecrement => Err(TypeError::inference(
                FrontendMessage::UnsupportedIncrementOperator,
            )),
            UnaryOperator::BitwiseNot => {
                if self.is_integer_primitive(&base_type) {
                    Ok(base_type)
                } else {
                    Err(crate::type_mismatch!(
                        "integer type",
                        format!("{:?}", base_type)
                    ))
                }
            }
        }
    }
}

// Large cast literals use the AST's i64 payload before storage normalisation.
fn cast_literal_source_type(expression: &Expression, inferred: Type) -> Type {
    if inferred != Type::I32 {
        return inferred;
    }

    let value = match &expression.kind {
        ExpressionKind::Integer(value) => Some(i128::from(*value)),
        ExpressionKind::UnaryOperator {
            operator: UnaryOperator::Negate,
            expression,
        } => match expression.kind {
            ExpressionKind::Integer(value) => Some(-i128::from(value)),
            _ => None,
        },
        _ => None,
    };

    if value.is_some_and(|value| value < i128::from(i32::MIN) || value > i128::from(i32::MAX)) {
        Type::I64
    } else {
        inferred
    }
}

fn unresolved_integer_literal(expression: &Expression) -> bool {
    if expression.value_type != Type::Unknown {
        return false;
    }

    match &expression.kind {
        ExpressionKind::Integer(_) => true,
        ExpressionKind::UnaryOperator {
            operator: UnaryOperator::Negate,
            expression: inner,
        } => inner.value_type == Type::Unknown && matches!(inner.kind, ExpressionKind::Integer(_)),
        _ => false,
    }
}
