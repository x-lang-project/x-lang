use super::{BorrowCheckError, BorrowChecker, ReferenceValue};
use x_ast::{Expression, ExpressionKind, Type};

impl BorrowChecker {
    pub(super) fn check_assignment(
        &mut self,
        target: &Expression,
        value: &Expression,
    ) -> Result<(), BorrowCheckError> {
        let references = self.check_transferred_value(value)?;

        if self.flow.path_terminated {
            return Ok(());
        }

        self.validate_borrowed_store_regions(target, &references)?;
        self.validate_reference_store_lifetime(target, &references)?;
        if let ExpressionKind::ArrayAccess { index, .. } = &target.kind {
            self.check_expression(index)?;
        }

        if let Some((owner, _)) = self.resolve_lvalue_owner_expression(target)
            && let Some((_, receiver_region)) = self.lookup_variable(&owner)
        {
            self.retain_reference_loans(&references, receiver_region);
        }

        match &target.kind {
            _ => Err(BorrowCheckError::InvalidAssignmentLValue),
        }
    }

    fn assign_array(
        &mut self,
        target: &Expression,
        array: &Expression,
        references: &ReferenceValue,
    ) -> Result<(), BorrowCheckError> {
        if let Some((owner_name, field_path)) = self.resolve_array_owner_expression(array) {
            let (variable_info, _) = self
                .lookup_variable_mut(&owner_name)
                .ok_or_else(|| BorrowCheckError::UndeclaredVariable(owner_name.clone()))?;

            if variable_info.is_moved {
                return Err(BorrowCheckError::UseAfterMove(owner_name.clone()));
            }

            if !variable_info.borrows.is_empty() {
                return Err(BorrowCheckError::MoveWhileBorrowed(owner_name.clone()));
            }

            if variable_info.field_borrows.iter().any(|(borrowed, loans)| {
                !loans.is_empty()
                    && field_path
                        .as_deref()
                        .is_none_or(|field| super::storage::field_paths_overlap(field, borrowed))
            }) {
                return Err(BorrowCheckError::MutableBorrowViolation(owner_name.clone()));
            }

            variable_info.is_initialised = true;
            self.update_projected_storage(target, references)?;

            Ok(())
        } else {
            self.check_expression(target)
        }
    }

    fn assign_referent(
        &mut self,
        target: &Expression,
        inner_expression: &Expression,
        references: &ReferenceValue,
    ) -> Result<(), BorrowCheckError> {
        if let ExpressionKind::Identifier(pointer_name) = &inner_expression.kind {
            let (variable_info, _) = self
                .lookup_variable_mut(pointer_name)
                .ok_or_else(|| BorrowCheckError::UndeclaredVariable(pointer_name.clone()))?;

            if variable_info.is_moved {
                return Err(BorrowCheckError::UseAfterMove(pointer_name.clone()));
            }

            match &variable_info.value_type {
                Type::Reference { is_mut, .. } | Type::RawPointer { is_mut, .. } => {
                    if !*is_mut {
                        return Err(BorrowCheckError::ImmutableBorrowViolation(
                            pointer_name.clone(),
                        ));
                    }
                }
                Type::Unknown => {}
                _ => {
                    return Err(BorrowCheckError::NonLValueReference(
                        inner_expression.kind.clone(),
                    ));
                }
            }

            if !variable_info.borrows.is_empty() {
                return Err(BorrowCheckError::MoveWhileBorrowed(pointer_name.clone()));
            }

            variable_info.is_initialised = true;
            self.update_referent_value(inner_expression, references)?;

            Ok(())
        } else {
            self.check_expression(target)?;

            self.update_referent_value(inner_expression, references)
        }
    }
}
