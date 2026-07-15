use super::{
    BorrowCheckError, BorrowChecker, FunctionLifetimeContext, MAX_AGGREGATE_LAYOUT_DEPTH,
    ReferenceValue, Region, parameter_origins,
};
use crate::ReferenceProjection;
use traversal::{LifetimeTraversal, OriginRequirement};
use x_ast::{Expression, ExpressionKind, Lifetime, Type};
use x_diagnostics::borrow_messages;

mod traversal;

impl BorrowChecker {
    pub(super) fn is_reference_expression(
        &self,
        current_expression: &Expression,
    ) -> Result<bool, BorrowCheckError> {
        if self.is_safe_reference_value(current_expression) {
            return Ok(true);
        }

        match &current_expression.kind {
            ExpressionKind::AddressOf { .. } => Ok(true),
            ExpressionKind::Try { expression } => self.is_reference_expression(expression),
            _ => Ok(false),
        }
    }

    pub(super) fn validate_reference_return(
        &self,
        expression: &Expression,
        references: &ReferenceValue,
    ) -> Result<(), BorrowCheckError> {
        let Some(current_context) = self.context.function_contexts.last() else {
            return Err(BorrowCheckError::InvalidReferenceReturn);
        };

        let return_type = &current_context.return_type;
        let Type::Reference {
            lifetime: return_lifetime,
            ..
        } = return_type
        else {
            return Err(BorrowCheckError::InvalidReferenceReturn);
        };

        match return_lifetime {
            Some(Lifetime::Static) => {
                if self.return_expression_has_lifetime(
                    expression,
                    &Lifetime::Static,
                    current_context,
                    references,
                )? {
                    Ok(())
                } else {
                    Err(BorrowCheckError::InvalidReferenceReturn)
                }
            }
            None | Some(Lifetime::Erased) | Some(Lifetime::Infer(_)) => {
                if Self::return_value_is_parameter_reference(current_context, references) {
                    Ok(())
                } else {
                    Err(BorrowCheckError::InvalidReferenceReturn)
                }
            }
            Some(expected) => {
                if self.return_expression_has_lifetime(
                    expression,
                    expected,
                    current_context,
                    references,
                )? {
                    Ok(())
                } else {
                    Err(BorrowCheckError::LifetimeTooShort {
                        target: self
                            .reference_target_name(expression)
                            .unwrap_or_else(|| "return value".to_string()),
                    })
                }
            }
        }
    }

    pub(super) fn validate_reference_store_lifetime(
        &self,
        target: &Expression,
        references: &ReferenceValue,
    ) -> Result<(), BorrowCheckError> {
        let requirement = if self.borrowed_receiver(target).is_some() {
            OriginRequirement::Known
        } else {
            OriginRequirement::DeclaredLifetime
        };

        self.validate_nested_lifetimes(
            &self.place_type(target),
            references,
            &self
                .reference_target_name(target)
                .unwrap_or_else(|| "stored reference".into()),
            LifetimeTraversal::new(requirement),
        )
    }

    pub(super) fn validate_value_lifetimes(
        &self,
        value_type: &Type,
        value: &ReferenceValue,
        target: &str,
    ) -> Result<(), BorrowCheckError> {
        self.validate_nested_lifetimes(
            value_type,
            value,
            target,
            LifetimeTraversal::new(OriginRequirement::DeclaredLifetime),
        )
    }

    pub(super) fn validate_returned_value(
        &self,
        value_type: &Type,
        value: &ReferenceValue,
    ) -> Result<(), BorrowCheckError> {
        self.validate_nested_lifetimes(
            value_type,
            value,
            "return value",
            LifetimeTraversal::new(OriginRequirement::External),
        )
    }

    pub(super) fn validate_tail_argument(
        &self,
        value_type: &Type,
        value: &ReferenceValue,
    ) -> Result<(), BorrowCheckError> {
        if let Some(origin) = value.find_target(|origin| origin.origin_region != Region(0)) {
            return Err(BorrowCheckError::LifetimeTooShort {
                target: origin.target_variable.clone(),
            });
        }

        self.validate_nested_lifetimes(
            value_type,
            value,
            "tail-call argument",
            LifetimeTraversal::new(OriginRequirement::TailTransfer),
        )
    }

    fn return_expression_has_lifetime(
        &self,
        expression: &Expression,
        expected: &Lifetime,
        context: &FunctionLifetimeContext,
        value: &ReferenceValue,
    ) -> Result<bool, BorrowCheckError> {
        let value_type = self
            .returned_reference_type(expression)?
            .unwrap_or_else(|| expression.value_type.clone());

        Ok(type_reference_lifetime_satisfies(&value_type, expected)
            && parameter_origins_satisfy(value, context, expected))
    }

    fn return_value_is_parameter_reference(
        context: &FunctionLifetimeContext,
        value: &ReferenceValue,
    ) -> bool {
        value.origin_known
            && !value.targets.is_empty()
            && value.targets.iter().all(|target| {
                target.origin_region == Region(0)
                    && context
                        .parameter_references
                        .contains_key(&target.target_variable)
            })
    }

    fn returned_reference_type(
        &self,
        expression: &Expression,
    ) -> Result<Option<Type>, BorrowCheckError> {
        match &expression.kind {
            ExpressionKind::Try { expression } => self.returned_reference_type(expression),
            ExpressionKind::Identifier(name) => self
                .lookup_variable(name)
                .map(|(binding, _)| Some(binding.value_type.clone()))
                .ok_or_else(|| BorrowCheckError::UndeclaredVariable(name.clone())),
            ExpressionKind::AddressOf { expression, .. } => {
                let Some((owner, Some(_))) = self.resolve_lvalue_owner_expression(expression)
                else {
                    // A parameter's address refers to its local stack slot.
                    return Ok(None);
                };

                Ok(self
                    .lookup_variable(&owner)
                    .filter(|(binding, _)| matches!(binding.value_type, Type::Reference { .. }))
                    .map(|(binding, _)| binding.value_type.clone()))
            }
            ExpressionKind::FieldAccess { .. } | ExpressionKind::ArrayAccess { .. } => {
                Ok(Some(self.place_type(expression)))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn reference_target_name(&self, expression: &Expression) -> Option<String> {
        match &expression.kind {
            ExpressionKind::Identifier(name) => Some(name.clone()),
            ExpressionKind::AddressOf { expression, .. } => self
                .resolve_lvalue_owner_expression(expression)
                .map(|(name, _)| name),
            ExpressionKind::Try { expression } => self.reference_target_name(expression),
            _ => None,
        }
    }
}

fn parameter_origins_satisfy(
    value: &ReferenceValue,
    context: &FunctionLifetimeContext,
    expected: &Lifetime,
) -> bool {
    value.origin_known
        && !value.targets.is_empty()
        && value.targets.iter().all(|target| {
            target.origin_region == Region(0)
                && context
                    .parameter_references
                    .get(&target.target_variable)
                    .is_some_and(|value_type| {
                        type_reference_lifetime_satisfies(value_type, expected)
                    })
        })
}

fn type_reference_lifetime_satisfies(value_type: &Type, expected: &Lifetime) -> bool {
    match value_type {
        Type::Reference { lifetime, .. } => match (lifetime, expected) {
            (Some(Lifetime::Static), _) => true,
            (Some(Lifetime::Named(actual)), Lifetime::Named(expected)) => actual == expected,
            (Some(Lifetime::Erased), _) | (Some(Lifetime::Infer(_)), _) => true,
            (None, Lifetime::Erased | Lifetime::Infer(_)) => true,
            _ => false,
        },
        _ => false,
    }
}
