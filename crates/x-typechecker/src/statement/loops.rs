use super::*;

impl TypeChecker {
    pub(super) fn check_range_bounds(
        &mut self,
        start: &mut Expression,
        end: &mut Expression,
        is_parallel: bool,
    ) -> Result<Type, TypeError> {
        let start_type = self.check_expression(start)?;
        let current_end_type = self.check_expression(end)?;

        for bound_type in [&start_type, &current_end_type] {
            if !self.is_integer_primitive(bound_type) {
                return Err(crate::type_mismatch!(
                    "integer range bound",
                    bound_type.to_string()
                ));
            }
            // Parallel workers use an i32 range ABI.
            if is_parallel && self.resolve(bound_type) != Type::I32 {
                return Err(crate::type_mismatch!(
                    "i32 parallel range bound",
                    bound_type.to_string()
                ));
            }
        }

        let mut range_type = self.resolve(&start_type);
        let end_type = self.resolve(&current_end_type);

        if range_type != end_type {
            if crate::expression::is_integer_literal_for_context(start) {
                range_type = end_type;
                start.value_type = range_type.clone();
            } else if crate::expression::is_integer_literal_for_context(end) {
                end.value_type = range_type.clone();
            } else {
                return Err(crate::type_mismatch!(
                    range_type.to_string(),
                    end_type.to_string()
                ));
            }
        }
        start.value_type = range_type.clone();
        end.value_type = range_type.clone();

        Ok(range_type)
    }
}
