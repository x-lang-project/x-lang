mod types;

use crate::{
    BorrowCheckError, BorrowChecker, ReferenceProjection, ReferenceTarget, ReferenceValue,
};
use crate::{Region, VariableInfo};
use types::stored_children;
use x_ast::{Expression, Type};

struct UniqueReborrow {
    referents: Vec<ReferenceTarget>,
    sources: Vec<ReferenceTarget>,
}

impl BorrowChecker {
    pub(crate) fn validate_unique_reborrow(
        &self,
        place: &Expression,
    ) -> Result<(), BorrowCheckError> {
        let Some(mut pointer) = self.borrowed_receiver(place) else {
            return Ok(());
        };

        let (referents, known) = self.address_reference_targets(place, true)?;
        let name = self
            .resolve_lvalue_owner_expression(pointer)
            .map(|(name, _)| name)
            .unwrap_or_else(|| "unique reborrow".into());
        // Input references may have aliases retained by the caller.
        if !known
            || referents.is_empty()
            || referents
                .iter()
                .any(|target| target.origin_region == Region(0) && !self.origin_is_unique(target))
        {
            return Err(BorrowCheckError::MutableBorrowViolation(name));
        }

        let mut request = UniqueReborrow {
            referents,
            sources: Vec::new(),
        };

        loop {
            let (sources, known) = self.address_reference_targets(pointer, false)?;

            if !known || sources.is_empty() {
                return Err(BorrowCheckError::MutableBorrowViolation(name));
            }

            request.sources.extend(sources);
            let Some(receiver) = self.borrowed_receiver(pointer) else {
                break;
            };

            pointer = receiver;
        }

        for scope in &self.flow.scopes {
            for (binding, storage) in scope.variables.iter() {
                let origin = ReferenceTarget {
                    binding_generation: scope.generation(binding),
                    projection: Vec::new(),
                    target_variable: binding.clone(),
                    origin_region: scope.region,
                    is_mut: false,
                };

                if self.storage_has_unique_alias(storage, origin, &request) {
                    return Err(BorrowCheckError::MutableBorrowViolation(name));
                }
            }

            for (binding, generations) in scope.shadowed.iter() {
                for (generation, storage) in generations.iter().enumerate() {
                    let origin = ReferenceTarget {
                        binding_generation: generation,
                        projection: Vec::new(),
                        target_variable: binding.clone(),
                        origin_region: scope.region,
                        is_mut: false,
                    };

                    if self.storage_has_unique_alias(storage, origin, &request) {
                        return Err(BorrowCheckError::MutableBorrowViolation(name));
                    }
                }
            }
        }

        Ok(())
    }

    fn storage_has_unique_alias(
        &self,
        storage: &VariableInfo,
        place: ReferenceTarget,
        request: &UniqueReborrow,
    ) -> bool {
        if storage.is_moved || !storage.is_initialised {
            return false;
        }
        // Callable declarations use Unknown and carry no stored references.
        if place.origin_region == Region(0) && storage.value_type == Type::Unknown {
            return false;
        }

        AliasSearch {
            checker: self,
            storage,
            place,
            request,
        }
        .find(&storage.references, &storage.value_type, 0)
    }

    fn origin_is_unique(&self, target: &ReferenceTarget) -> bool {
        self.context
            .function_contexts
            .last()
            .is_some_and(|context| {
                matches!(
                    context.parameter_references.get(&target.target_variable),
                    Some(Type::Reference {
                        is_unique: true,
                        ..
                    })
                )
            })
    }
}

struct AliasSearch<'a> {
    checker: &'a BorrowChecker,
    storage: &'a VariableInfo,
    place: ReferenceTarget,
    request: &'a UniqueReborrow,
}

impl AliasSearch<'_> {
    fn find(&mut self, value: &ReferenceValue, value_type: &Type, depth: usize) -> bool {
        if self.storage.borrows.iter().any(|loan| loan.is_mut)
            || self.storage.field_borrows.iter().any(|(field, loans)| {
                let frozen = ReferenceProjection::from_field_path(
                    &self.storage.value_type,
                    field,
                    &self.checker.context.struct_layouts,
                );

                loans.iter().any(|loan| loan.is_mut)
                    && frozen.len() <= self.place.projection.len()
                    && projections_overlap(&frozen, &self.place.projection)
            })
        {
            return false;
        }

        if self.storage.moved_fields.iter().any(|field| {
            let moved = ReferenceProjection::from_field_path(
                &self.storage.value_type,
                field,
                &self.checker.context.struct_layouts,
            );

            moved.len() <= self.place.projection.len()
                && projections_overlap(&moved, &self.place.projection)
        }) {
            return false;
        }

        let source = self.request.sources.iter().any(|source| {
            same_storage(source, &self.place) && source.projection == self.place.projection
        });

        if !source && matches!(value_type, Type::Reference { .. }) && !value.origin_known {
            return true;
        }

        if !source
            && value.targets.iter().any(|alias| {
                self.request.referents.iter().any(|target| {
                    same_storage(alias, target)
                        && projections_overlap(&alias.projection, &target.projection)
                }) || self.request.sources.iter().any(|target| {
                    same_storage(alias, target)
                        && projections_overlap(&alias.projection, &target.projection)
                })
            })
        {
            return true;
        }

        if depth >= crate::MAX_AGGREGATE_LAYOUT_DEPTH {
            return true;
        }

        let Some(children) =
            stored_children(value_type, value, &self.checker.context.struct_layouts)
        else {
            return true;
        };

        let unknown = ReferenceValue::default();

        for (field, value_type) in children {
            let child = value.fields.get(&field).unwrap_or(&unknown);
            self.place.projection.push(field.clone());
            let conflict = self.find(child, &value_type, depth + 1);
            self.place.projection.pop();

            if conflict {
                return true;
            }
        }

        false
    }
}

fn same_storage(left: &ReferenceTarget, right: &ReferenceTarget) -> bool {
    left.origin_region == right.origin_region
        && left.binding_generation == right.binding_generation
        && left.target_variable == right.target_variable
}

fn projections_overlap(left: &[ReferenceProjection], right: &[ReferenceProjection]) -> bool {
    left.iter().zip(right).all(|(left, right)| {
        left == right
            || matches!(left, ReferenceProjection::AnyIndex)
            || matches!(right, ReferenceProjection::AnyIndex)
    })
}
