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
}
