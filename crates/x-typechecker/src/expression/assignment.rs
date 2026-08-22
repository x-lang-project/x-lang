use super::*;

impl TypeChecker {
    pub(super) fn check_assignment_expression(
        &mut self,
        target: &mut Expression,
        value: &mut Expression,
    ) -> Result<Type, TypeError> {
        // Reject nonlocal stores before pure lookup treats the target as a read.
        if self.current_function_is_pure()
            && matches!(target.kind, ExpressionKind::Identifier(_))
            && !self.is_local_owned_assignment(target)
        {
            self.require_impure_operation(ImpureOperation::Assignment)?;
        }

        let target_type = self.check_expression(target)?;

        if self.current_function_is_pure() && !self.is_local_owned_assignment(target) {
            self.require_impure_operation(ImpureOperation::Assignment)?;
        }

        if let ExpressionKind::Identifier(name) = &target.kind
            && self
                .function_contexts
                .last()
                .is_some_and(|context| context.immutable_parameters.contains(name))
        {
            return Err(TypeError::ImmutableParameterAssignment { name: name.clone() });
        }

        let mut value_type = self.check_expression(value)?;

        if is_integer_literal(value) && self.is_integer_primitive(&target_type) {
            value.value_type = target_type.clone();
            value_type = target_type.clone();
        }

        self.unify(&target_type, &value_type)?;
        self.update_local_reference_assignment(target, value);

        Ok(Type::Void)
    }
}
