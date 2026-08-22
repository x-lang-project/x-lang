use crate::{TypeChecker, errors::TypeError};
use x_ast::{Expression, ExpressionKind, Type};

impl TypeChecker {
    pub(in crate::expression) fn method_receiver_argument(
        &mut self,
        object: &Expression,
        parameter: &Type,
    ) -> Result<Expression, TypeError> {
        let receiver = object.clone();
        let Type::Reference {
            is_mut, is_unique, ..
        } = parameter
        else {
            return Ok(receiver);
        };

        if matches!(receiver.value_type, Type::Reference { .. }) {
            return Ok(receiver);
        }

        self.validate_borrowed_place(&receiver, *is_mut)?;
        let span = receiver.span;
        let mut argument = Expression::new(
            parameter.clone(),
            ExpressionKind::AddressOf {
                is_mut: *is_mut,
                is_unique: *is_unique,
                expression: Box::new(receiver),
            },
        );
        argument.span = span;

        Ok(argument)
    }
}
