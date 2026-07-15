use super::Region;
use super::{BorrowCheckError, BorrowChecker, ReferenceValue};
use x_ast::Expression;

impl BorrowChecker {
    pub(super) fn check_call_value(
        &mut self,
        name: &str,
        arguments: &[Expression],
    ) -> Result<ReferenceValue, BorrowCheckError> {
        if x_ast::intrinsics::is_catch_call(name)
            && self.lookup_variable(name).is_none()
            && let [call, fallback] = arguments
        {
            let receiver = self.current_region()?;

            return self.with_scope(|this| {
                let success = this.check_value_expression(call)?;
                let success_state = this.clone();
                let mut failure_state = this.clone();
                let failure = failure_state.check_transferred_value(fallback)?;
                this.merge_branch_states(&success_state, &failure_state);
                let result = success.join(&failure);
                this.retain_reference_loans(&result, receiver);

                Ok(result)
            });
        }

        self.check_call_value_inner(name, arguments, false)
    }

    pub(super) fn check_tail_call(
        &mut self,
        name: &str,
        arguments: &[Expression],
    ) -> Result<(), BorrowCheckError> {
        self.check_call_value_inner(name, arguments, true)
            .map(|_| ())
    }

    fn check_call_value_inner(
        &mut self,
        name: &str,
        arguments: &[Expression],
        tail: bool,
    ) -> Result<ReferenceValue, BorrowCheckError> {
        let receiver_region = self.current_region()?;

        self.with_scope(|this| {
            let mut values = Vec::with_capacity(arguments.len());

            for argument in arguments {
                let value = this.check_transferred_value(argument)?;

                if this.flow.path_terminated {
                    return Ok(ReferenceValue::default());
                }

                if tail {
                    this.validate_tail_argument(&this.place_type(argument), &value)?;
                }

                values.push(value);
            }

            let result = this
                .lookup_variable(name)
                .filter(|(_, region)| *region == Region(0))
                .and_then(|_| this.context.return_summaries.get(name))
                .map(|summary| summary.instantiate(&values, arguments))
                .unwrap_or_default();
            this.retain_reference_loans(&result, receiver_region);

            Ok(result)
        })
    }
}
