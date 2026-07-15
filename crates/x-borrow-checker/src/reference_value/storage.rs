use std::collections::HashMap;

use super::ReferenceValue;
use crate::ReferenceProjection;
use crate::{BorrowCheckError, BorrowChecker, Region};
use x_ast::{Expression, ExpressionKind};

impl BorrowChecker {
    pub(super) fn reference_value(
        &self,
        expression: &Expression,
    ) -> Result<ReferenceValue, BorrowCheckError> {
        match &expression.kind {
            ExpressionKind::Identifier(name) => Ok(self
                .lookup_variable(name)
                .map(|(binding, _)| binding.references.clone())
                .unwrap_or_default()),
            ExpressionKind::Dereference {
                expression: pointer,
            } => Ok(self.read_referent_value(&self.stored_place_references(pointer)?)),
            ExpressionKind::AddressOf {
                expression, is_mut, ..
            } => {
                let (targets, origin_known) =
                    self.address_reference_targets(expression, *is_mut)?;

                Ok(ReferenceValue {
                    targets,
                    origin_known,
                    fields: HashMap::new(),
                    is_unreachable: false,
                })
            }
            _ => Ok(ReferenceValue::default()),
        }
    }

    pub(super) fn projected_object_value(
        &self,
        object: &Expression,
        value: ReferenceValue,
    ) -> ReferenceValue {
        if self.is_safe_reference_value(object) {
            self.read_referent_value(&value)
        } else {
            value
        }
    }

    pub(crate) fn update_projected_storage(
        &mut self,
        target: &Expression,
        value: &ReferenceValue,
    ) -> Result<(), BorrowCheckError> {
        let (targets, origin_known) = self.address_reference_targets(target, true)?;
        self.update_storage_targets(
            ReferenceValue {
                targets,
                origin_known,
                fields: HashMap::new(),
                is_unreachable: false,
            },
            value,
        );

        Ok(())
    }

    fn read_referent_value(&self, pointer: &ReferenceValue) -> ReferenceValue {
        if !pointer.origin_known {
            return ReferenceValue::default();
        }

        pointer
            .targets
            .iter()
            .map(|target| {
                self.flow
                    .scopes
                    .iter()
                    .find(|scope| scope.region == target.origin_region && scope.region != Region(0))
                    .and_then(|scope| {
                        scope.storage(&target.target_variable, target.binding_generation)
                    })
                    .map(|binding| binding.references.project(&target.projection))
                    .unwrap_or_default()
            })
            .reduce(|first_value, second_value| first_value.join(&second_value))
            .unwrap_or_default()
    }

    pub(crate) fn update_referent_value(
        &mut self,
        expression: &Expression,
        value: &ReferenceValue,
    ) -> Result<(), BorrowCheckError> {
        if !self.is_safe_reference_value(expression) {
            return Ok(());
        }

        let destinations = self.stored_place_references(expression)?;
        self.update_storage_targets(destinations, value);

        Ok(())
    }

    fn update_storage_targets(&mut self, destinations: ReferenceValue, value: &ReferenceValue) {
        let exact = destinations.origin_known && destinations.targets.len() == 1;

        for target in destinations.targets {
            if target.origin_region == Region(0) {
                continue;
            }

            self.retain_reference_loans(value, target.origin_region);

            if let Some(binding) = self
                .flow
                .scopes
                .iter_mut()
                .find(|scope| scope.region == target.origin_region)
                .and_then(|scope| {
                    scope.storage_mut(&target.target_variable, target.binding_generation)
                })
            {
                let replacement = if exact {
                    value.clone()
                } else {
                    binding.references.project(&target.projection).join(value)
                };

                binding.references.assign(&target.projection, replacement);
                binding.is_initialised = true;
            }
        }
    }

    pub(crate) fn borrowed_receiver<'a>(&self, target: &'a Expression) -> Option<&'a Expression> {
        match &target.kind {
            ExpressionKind::Dereference { expression }
                if self.is_safe_reference_value(expression) =>
            {
                Some(expression)
            }
            ExpressionKind::FieldAccess { object, .. }
            | ExpressionKind::ArrayAccess { array: object, .. } => {
                if self.is_safe_reference_value(object) {
                    Some(object)
                } else {
                    self.borrowed_receiver(object)
                }
            }
            _ => None,
        }
    }

    pub(crate) fn stored_place_references(
        &self,
        expression: &Expression,
    ) -> Result<ReferenceValue, BorrowCheckError> {
        match &expression.kind {
            ExpressionKind::FieldAccess { object, field } => {
                let value = self.stored_place_references(object)?;

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
                let value = self.stored_place_references(array)?;

                Ok(self
                    .projected_object_value(array, value)
                    .project(&[ReferenceProjection::from_index_expression(index)]))
            }
            _ => self.reference_value(expression),
        }
    }
}
