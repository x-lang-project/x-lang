mod methods;

use super::{BorrowCheckError, BorrowChecker, ReferenceValue};
use x_ast::SourceError;
use x_ast::{Expression, ExpressionKind, Type};

impl BorrowChecker {
    pub(super) fn check_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<(), BorrowCheckError> {
        if self.flow.path_terminated {
            return Ok(());
        }

        self.check_expression_inner(expression)
            .map_err(|error| error.at_source(expression.span))
    }

    fn check_expression_inner(&mut self, expression: &Expression) -> Result<(), BorrowCheckError> {
        match &expression.kind {
            ExpressionKind::Identifier(name) => self.use_ident(name),
            ExpressionKind::AddressOf {
                is_mut,
                is_unique,
                expression: inner_expression,
            } => {
                if *is_unique {
                    self.validate_unique_reborrow(inner_expression)?;
                }
                // Reborrows freeze the reference binding as well as its owner.
                if let Some(pointer) = self.borrowed_receiver(inner_expression)
                    && let Some((owner, field)) = self.resolve_reborrow_owner(pointer)
                {
                    self.check_place_indices(inner_expression)?;

                    return self.create_reborrow(&owner, field.as_deref(), *is_mut || *is_unique);
                }

                match &inner_expression.kind {
                    ExpressionKind::Identifier(tgt) => {
                        self.create_borrow(tgt, None, *is_mut || *is_unique)
                    }
                    ExpressionKind::FieldAccess { .. } => {
                        self.check_place_indices(inner_expression)?;

                        if let Some((owner, field)) =
                            self.resolve_lvalue_owner_expression(inner_expression)
                        {
                            self.create_borrow(&owner, field.as_deref(), *is_mut || *is_unique)
                        } else {
                            Err(BorrowCheckError::NonLValueReference(
                                inner_expression.kind.clone(),
                            ))
                        }
                    }
                    ExpressionKind::ArrayAccess { array, index } => {
                        self.check_expression(index)?;
                        self.check_place_indices(array)?;

                        if let Some((owner_name, field)) =
                            self.resolve_array_owner_expression(array)
                        {
                            self.create_borrow(&owner_name, field.as_deref(), *is_mut || *is_unique)
                        } else {
                            Ok(())
                        }
                    }
                    ExpressionKind::Dereference {
                        expression: pointer_expression,
                    } => {
                        self.check_place_indices(pointer_expression)?;

                        if self.is_safe_reference_value(pointer_expression)
                            && let Some((owner, field)) =
                                self.resolve_lvalue_owner_expression(pointer_expression)
                        {
                            self.create_reborrow(&owner, field.as_deref(), *is_mut || *is_unique)
                        } else {
                            self.check_expression(pointer_expression)
                        }
                    }
                    _ => Err(BorrowCheckError::NonLValueReference(
                        inner_expression.kind.clone(),
                    )),
                }
            }

            ExpressionKind::Block { statements, result } => self
                .check_block_value(statements, result.as_deref())
                .map(|_| ()),
            ExpressionKind::If {
                condition,
                then_block,
                else_block,
            } => self
                .check_if(condition, then_block, else_block.as_deref(), true)
                .map(|_| ()),
            ExpressionKind::Match { scrutinee, arms } => {
                self.check_match_value(scrutinee, arms).map(|_| ())
            }
            ExpressionKind::FunctionCall {
                target: name,
                arguments,
            } => self
                .check_call_value(&name.symbol_name(), arguments)
                .map(|_| ()),
            ExpressionKind::FieldAccess { object, .. } => self.check_place_read(expression, object),
            ExpressionKind::ArrayAccess { array, index } => {
                self.check_expression(index)?;

                self.check_place_read(expression, array)
            }
            ExpressionKind::Array(elements) | ExpressionKind::Tuple(elements) => {
                for expression_value in elements {
                    self.check_expression(expression_value)?;
                }

                Ok(())
            }
            ExpressionKind::ArrayRepeat {
                value: left,
                count: right,
            }
            | ExpressionKind::BinaryOperator { left, right, .. } => {
                self.check_expression(left)?;

                self.check_expression(right)
            }
            ExpressionKind::Dereference { expression: inner }
            | ExpressionKind::UnaryOperator {
                expression: inner, ..
            }
            | ExpressionKind::Try { expression: inner } => self.check_expression(inner),
            ExpressionKind::Integer(_)
            | ExpressionKind::Float(_)
            | ExpressionKind::Boolean(_)
            | ExpressionKind::String(_)
            | ExpressionKind::Character(_)
            | ExpressionKind::TypeLiteral(_)
            | ExpressionKind::SizeOf(_) => Ok(()),
            ExpressionKind::Cast {
                expression: inner,
                to,
            } => {
                if matches!(to, Type::RawPointer { .. })
                    && let ExpressionKind::AddressOf {
                        expression: place, ..
                    } = &inner.kind
                {
                    return self.check_raw_address_place(place);
                }

                self.check_expression(inner)
            }
            ExpressionKind::Assignment { target, value } => self.check_assignment(target, value),
            ExpressionKind::StructInstantiate(initialise) => {
                for (_, field_expression) in &initialise.fields {
                    self.check_expression(field_expression)?;
                }

                Ok(())
            }
            ExpressionKind::AnonymousFunction { .. } | ExpressionKind::Assembly { .. } => Ok(()),
            ExpressionKind::MethodCall {
                object,
                target: method,
                arguments,
            } => self
                .check_method_value(object, &method.symbol_name(), arguments)
                .map(|_| ()),
        }
    }

    fn check_raw_address_place(&mut self, place: &Expression) -> Result<(), BorrowCheckError> {
        match &place.kind {
            ExpressionKind::Identifier(name) => self
                .lookup_variable(name)
                .map(|_| ())
                .ok_or_else(|| BorrowCheckError::UndeclaredVariable(name.clone())),
            ExpressionKind::FieldAccess { object, .. } => self.check_raw_address_place(object),
            ExpressionKind::ArrayAccess { array, index } => {
                self.check_raw_address_place(array)?;

                self.check_expression(index)
            }
            ExpressionKind::Dereference { expression } => self.check_expression(expression),
            _ => Err(BorrowCheckError::NonLValueReference(place.kind.clone())),
        }
    }

    fn resolve_reborrow_owner(
        &self,
        current_expression: &Expression,
    ) -> Option<(String, Option<String>)> {
        self.resolve_lvalue_owner_expression(current_expression)
            .or_else(|| match &current_expression.kind {
                ExpressionKind::Dereference { expression }
                    if self.is_safe_reference_value(expression) =>
                {
                    self.resolve_reborrow_owner(expression)
                }
                _ => None,
            })
    }

    fn check_place_read(
        &mut self,
        place: &Expression,
        base: &Expression,
    ) -> Result<(), BorrowCheckError> {
        if let Some((owner, field)) = self.resolve_lvalue_owner_expression(place) {
            self.check_place_indices(base)?;
            let (storage, _) = self
                .lookup_variable(&owner)
                .ok_or_else(|| BorrowCheckError::UndeclaredVariable(owner.clone()))?;

            storage.check_read(&owner, field.as_deref())
        } else {
            self.check_expression(base)
        }
    }

    fn check_place_indices(&mut self, place: &Expression) -> Result<(), BorrowCheckError> {
        match &place.kind {
            ExpressionKind::ArrayAccess { array, index } => {
                self.check_place_indices(array)?;

                self.check_expression(index)
            }
            ExpressionKind::FieldAccess { object, .. } => self.check_place_indices(object),
            _ => Ok(()),
        }
    }
}
