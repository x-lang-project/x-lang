use super::{BorrowCheckError, BorrowChecker, ReferenceTarget};
use crate::ReferenceProjection;
use x_ast::{Expression, ExpressionKind};

impl BorrowChecker {
    pub(super) fn resolve_array_owner_expression(
        &self,
        expression: &Expression,
    ) -> Option<(String, Option<String>)> {
        let owner = match &expression.kind {
            ExpressionKind::ArrayAccess { array, .. } => array.as_ref(),
            _ => expression,
        };

        self.resolve_lvalue_owner_expression(owner)
    }

    pub(super) fn resolve_lvalue_owner_expression(
        &self,
        expression: &Expression,
    ) -> Option<(String, Option<String>)> {
        match &expression.kind {
            ExpressionKind::Identifier(name) => Some((name.clone(), None)),
            ExpressionKind::FieldAccess { object, field } => {
                let (owner, parent_field) = self.resolve_lvalue_owner_expression(object)?;
                let field_path = parent_field
                    .map(|parent| format!("{parent}.{field}"))
                    .unwrap_or_else(|| field.clone());

                Some((owner, Some(field_path)))
            }
            ExpressionKind::ArrayAccess { array, .. } => self.resolve_array_owner_expression(array),
            _ => None,
        }
    }

    pub(super) fn address_reference_targets(
        &self,
        expression: &Expression,
        is_mut: bool,
    ) -> Result<(Vec<ReferenceTarget>, bool), BorrowCheckError> {
        let (mut targets, known) = match &expression.kind {
            ExpressionKind::Dereference {
                expression: pointer,
            } => {
                let value = self.stored_place_references(pointer)?;

                (value.targets, value.origin_known)
            }
            ExpressionKind::FieldAccess { object, field } => {
                let (mut targets, known) = if self.is_safe_reference_value(object) {
                    let value = self.stored_place_references(object)?;

                    (value.targets, value.origin_known)
                } else {
                    self.address_reference_targets(object, is_mut)?
                };

                for target in &mut targets {
                    target
                        .projection
                        .push(ReferenceProjection::Field(field.clone()));
                }

                (targets, known)
            }
            ExpressionKind::ArrayAccess { array, index } => {
                let (mut targets, known) = if self.is_safe_reference_value(array) {
                    let value = self.stored_place_references(array)?;

                    (value.targets, value.origin_known)
                } else {
                    self.address_reference_targets(array, is_mut)?
                };

                let index = ReferenceProjection::from_index_expression(index);

                for target in &mut targets {
                    target.projection.push(index.clone());
                }

                (targets, known)
            }
            ExpressionKind::Identifier(owner) => {
                let (_, region) = self
                    .lookup_variable(owner)
                    .ok_or_else(|| BorrowCheckError::UndeclaredVariable(owner.clone()))?;

                (
                    vec![ReferenceTarget {
                        binding_generation: self
                            .flow
                            .scopes
                            .iter()
                            .find(|scope| scope.region == region)
                            .map(|scope| scope.generation(owner))
                            .unwrap_or_default(),
                        target_variable: owner.clone(),
                        origin_region: region,
                        is_mut,
                        projection: Vec::new(),
                    }],
                    true,
                )
            }
            _ => (Vec::new(), false),
        };

        for target in &mut targets {
            target.is_mut = is_mut;
        }

        Ok((targets, known))
    }
}
