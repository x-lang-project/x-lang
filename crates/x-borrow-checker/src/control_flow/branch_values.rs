use super::super::{BorrowCheckError, BorrowChecker, ReferenceValue, Region};
use crate::ReferenceProjection;
use x_ast::{Expression, MatchArm, Pattern, Statement, Type};

impl BorrowChecker {
    fn finish_branch_value(
        &mut self,
        value: ReferenceValue,
        receiver_region: Region,
    ) -> Result<Option<ReferenceValue>, BorrowCheckError> {
        if self.flow.path_terminated {
            return Ok(None);
        }

        let region = self.current_region()?;

        if value
            .find_target(|target| target.origin_region == region)
            .is_some()
        {
            return Err(BorrowCheckError::DanglingBranchReference);
        }

        self.retain_reference_loans(&value, receiver_region);

        Ok(Some(value))
    }
}

#[derive(Default)]
struct BranchValue(Option<ReferenceValue>);

impl BranchValue {
    fn include(&mut self, value: Option<ReferenceValue>) {
        if let Some(value) = value {
            self.0 = Some(
                self.0
                    .take()
                    .map(|previous| previous.join(&value))
                    .unwrap_or(value),
            );
        }
    }

    fn finish(self) -> ReferenceValue {
        self.0.unwrap_or_default()
    }
}
