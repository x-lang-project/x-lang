use super::ReferenceValue;
use crate::ReferenceProjection;
use crate::{BorrowCheckError, BorrowChecker, Region};
use x_ast::Expression;

impl BorrowChecker {
    pub(crate) fn validate_borrowed_store_regions(
        &self,
        target: &Expression,
        value: &ReferenceValue,
    ) -> Result<(), BorrowCheckError> {
        if value
            .find_target(|source| source.origin_region != Region(0))
            .is_none()
        {
            return Ok(());
        }

        let Some(pointer) = self.borrowed_receiver(target) else {
            return Ok(());
        };

        let destinations = self.stored_place_references(pointer)?;

        if let Some(escaping) = value.find_target(|source| {
            source.origin_region != Region(0)
                && (!destinations.origin_known
                    || destinations.targets.is_empty()
                    || destinations
                        .targets
                        .iter()
                        .any(|destination| source.origin_region.0 > destination.origin_region.0))
        }) {
            return Err(BorrowCheckError::LifetimeTooShort {
                target: escaping.target_variable.clone(),
            });
        }

        Ok(())
    }

    pub(crate) fn retain_reference_loans(
        &mut self,
        value: &ReferenceValue,
        receiver_region: Region,
    ) {
        // Escaping reborrows keep their reference binding frozen.
        for scope in &mut self.flow.scopes {
            if !scope.may_need_expiry {
                continue;
            }

            for binding in scope.values_mut() {
                if binding.references.shares_owner(value) {
                    for loan in &mut binding.borrows {
                        if loan.is_reborrow && loan.region.0 > receiver_region.0 {
                            loan.region = receiver_region;
                        }
                    }
                }

                for (field, loans) in &mut binding.field_borrows {
                    let path = ReferenceProjection::from_field_path(
                        &binding.value_type,
                        field,
                        &self.context.struct_layouts,
                    );

                    if binding.references.project(&path).shares_owner(value) {
                        for loan in loans {
                            if loan.is_reborrow && loan.region.0 > receiver_region.0 {
                                loan.region = receiver_region;
                            }
                        }
                    }
                }
            }
        }

        for reference in &value.targets {
            if let Some(referent) = self
                .flow
                .scopes
                .iter_mut()
                .find(|scope| scope.region == reference.origin_region)
                .and_then(|scope| {
                    scope.storage_mut(&reference.target_variable, reference.binding_generation)
                })
            {
                for loan in referent
                    .borrows
                    .iter_mut()
                    .chain(referent.field_borrows.values_mut().flatten())
                {
                    if loan.region.0 > receiver_region.0 {
                        loan.region = receiver_region;
                    }
                }
            }
        }

        for field in value.fields.values() {
            self.retain_reference_loans(field, receiver_region);
        }
    }
}
