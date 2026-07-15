use super::super::{BorrowCheckError, BorrowChecker, ReferenceValue, Region};
use crate::ReferenceProjection;
use x_ast::{Expression, MatchArm, Pattern, Statement, Type};

impl BorrowChecker {
    pub(crate) fn check_if(
        &mut self,
        condition: &Expression,
        then_block: &[Statement],
        else_block: Option<&[Statement]>,
        produces_value: bool,
    ) -> Result<ReferenceValue, BorrowCheckError> {
        self.check_expression(condition)?;
        let receiver_region = self.current_region()?;
        let mut then_checker = self.fork();
        let mut else_checker = self.fork();
        let mut result = BranchValue::default();

        for (checker, block) in [
            (&mut then_checker, then_block),
            (&mut else_checker, else_block.unwrap_or_default()),
        ] {
            let value = checker.with_scope(|this| {
                let value = if produces_value {
                    this.check_value_block(block)?
                } else {
                    this.check_statements(block)?;

                    ReferenceValue::default()
                };

                this.finish_branch_value(value, receiver_region)
            })?;

            result.include(value);
        }

        self.merge_branch_states(&then_checker, &else_checker);

        Ok(result.finish())
    }

    pub(crate) fn check_match_value(
        &mut self,
        scrutinee: &Expression,
        arms: &[MatchArm],
    ) -> Result<ReferenceValue, BorrowCheckError> {
        let scrutinee_value = self.check_value_expression(scrutinee)?;
        let before = self.flow.clone();
        let receiver_region = self.current_region()?;
        let mut merged = None;
        let mut result = BranchValue::default();

        for arm in arms {
            let mut checker = self.at_flow(before.clone());
            let value = checker.with_scope(|this| {
                match &arm.pattern {
                    Pattern::Identifier(name) => {
                        this.declare_variable(name, scrutinee.value_type.clone(), true)?;

                        if let Some((binding, _)) = this.lookup_variable_mut(name) {
                            binding.references = scrutinee_value.clone();
                        }
                    }
                    Pattern::Enum { bindings, .. } | Pattern::Tuple(bindings) => {
                        for (index, name) in bindings.iter().enumerate() {
                            this.declare_variable(name, Type::Unknown, true)?;

                            if let Some((binding, _)) = this.lookup_variable_mut(name) {
                                binding.references = scrutinee_value
                                    .fields
                                    .get(&ReferenceProjection::Index(index as i64))
                                    .cloned()
                                    .unwrap_or_default();
                            }
                        }
                    }
                    Pattern::Wildcard => {}
                }

                let value = if let Some(expression) = &arm.result {
                    this.check_statements(&arm.body)?;

                    this.check_transferred_value(expression)?
                } else {
                    this.check_value_block(&arm.body)?
                };

                this.finish_branch_value(value, receiver_region)
            })?;

            result.include(value);

            if let Some(previous) = merged {
                self.flow.merge_branches(&previous, &checker.flow);
                merged = Some(self.flow.clone());
            } else {
                merged = Some(checker.flow);
            }
        }

        if let Some(merged) = merged {
            self.flow.merge_branches(&merged, &merged);
        }

        Ok(result.finish())
    }

    pub(crate) fn check_block_value(
        &mut self,
        statements: &[Statement],
        expression: Option<&Expression>,
    ) -> Result<ReferenceValue, BorrowCheckError> {
        let receiver = self.current_region()?;

        self.with_scope(|this| {
            this.check_statements(statements)?;
            let value = match expression {
                Some(expression) => this.check_transferred_value(expression)?,
                None => ReferenceValue::default(),
            };

            Ok(this
                .finish_branch_value(value, receiver)?
                .unwrap_or_default())
        })
    }

    fn check_value_block(
        &mut self,
        block: &[Statement],
    ) -> Result<ReferenceValue, BorrowCheckError> {
        if let Some((Statement::Expression { expression }, prefix)) = block.split_last() {
            self.check_statements(prefix)?;

            self.check_transferred_value(expression)
        } else {
            self.check_statements(block)?;

            Ok(ReferenceValue::default())
        }
    }

    pub(crate) fn check_transferred_value(
        &mut self,
        expression: &Expression,
    ) -> Result<ReferenceValue, BorrowCheckError> {
        let value = self.check_value_expression(expression)?;

        if !self.flow.path_terminated {
            self.transfer_value(expression)?;
        }

        Ok(value)
    }

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
