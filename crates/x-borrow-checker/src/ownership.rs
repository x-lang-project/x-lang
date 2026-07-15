mod unique_aliases;

use super::{BorrowCheckError, BorrowChecker, ReferenceValue, Region, storage};
use crate::ReferenceProjection;
use x_ast::{Expression, ExpressionKind};

impl BorrowChecker {
    pub(super) fn create_borrow(
        &mut self,
        name: &str,
        field: Option<&str>,
        is_mut: bool,
    ) -> Result<(), BorrowCheckError> {
        let region = self.current_region()?;
        let (variable_info, _) = self
            .lookup_variable_mut(name)
            .ok_or_else(|| BorrowCheckError::UndeclaredVariable(name.to_string()))?;

        variable_info.create_borrow(name, field, region, is_mut)
    }

    pub(super) fn create_reborrow(
        &mut self,
        name: &str,
        field: Option<&str>,
        is_mut: bool,
    ) -> Result<(), BorrowCheckError> {
        self.create_borrow(name, field, is_mut)?;
        let (binding, _) = self
            .lookup_variable_mut(name)
            .ok_or_else(|| BorrowCheckError::UndeclaredVariable(name.into()))?;
        let loans = match field {
            Some(field) => binding.field_borrows.get_mut(field),
            None => Some(&mut binding.borrows),
        };

        if let Some(loan) = loans.and_then(|loans| loans.last_mut()) {
            loan.is_reborrow = true;
        }

        Ok(())
    }

    pub(super) fn move_ident(&mut self, name: &str) -> Result<(), BorrowCheckError> {
        let (variable_info, region) = self
            .lookup_variable_mut(name)
            .ok_or_else(|| BorrowCheckError::UndeclaredVariable(name.to_string()))?;

        if region == Region(0) {
            return Ok(());
        }

        variable_info.check_available(name)?;
        if !variable_info.borrows.is_empty()
            || variable_info
                .field_borrows
                .values()
                .any(|fb| !fb.is_empty())
        {
            return Err(BorrowCheckError::MoveWhileBorrowed(name.to_string()));
        }

        variable_info.is_moved = true;

        Ok(())
    }

    pub(super) fn transfer_value(
        &mut self,
        expression: &Expression,
    ) -> Result<(), BorrowCheckError> {
        if self.is_copy_type(&self.place_type(expression)) {
            return Ok(());
        }

        if let ExpressionKind::Identifier(name) = &expression.kind {
            return self.move_ident(name);
        }

        if let Some(pointer) = self.borrowed_receiver(expression) {
            return Err(BorrowCheckError::MoveWhileBorrowed(
                self.resolve_lvalue_owner_expression(pointer)
                    .map(|(owner, _)| owner)
                    .unwrap_or_else(|| "borrowed storage".into()),
            ));
        }

        if let Some((owner, field)) = self.resolve_lvalue_owner_expression(expression) {
            if let Some(field) = field {
                let projection = self
                    .lookup_variable(&owner)
                    .map(|(binding, _)| {
                        ReferenceProjection::from_field_path(
                            &binding.value_type,
                            &field,
                            &self.context.struct_layouts,
                        )
                    })
                    .unwrap_or_default();
                let (storage, _) = self
                    .lookup_variable_mut(&owner)
                    .ok_or_else(|| BorrowCheckError::UndeclaredVariable(owner.clone()))?;
                storage.check_place_available(&owner, Some(&field))?;
                if !storage.borrows.is_empty()
                    || storage.field_borrows.iter().any(|(borrowed, loans)| {
                        !loans.is_empty() && storage::field_paths_overlap(&field, borrowed)
                    })
                {
                    return Err(BorrowCheckError::MoveWhileBorrowed(format!(
                        "{owner}.{field}"
                    )));
                }

                storage.moved_fields.insert(field.clone());
                storage
                    .references
                    .assign(&projection, ReferenceValue::default());

                return Ok(());
            }

            return self.move_ident(&owner);
        }

        Ok(())
    }

    pub(super) fn use_ident(&mut self, name: &str) -> Result<(), BorrowCheckError> {
        let (variable_info, _) = self
            .lookup_variable_mut(name)
            .ok_or_else(|| BorrowCheckError::UndeclaredVariable(name.to_string()))?;

        variable_info.check_read(name, None)
    }
}
