use super::*;
use x_ast::intrinsics::{BuiltinCall, BuiltinKind, PointerOperation, SliceOperation};

impl BorrowChecker {
    pub(crate) fn check_method_value(
        &mut self,
        object: &Expression,
        method: &str,
        current_arguments: &[Expression],
    ) -> Result<ReferenceValue, BorrowCheckError> {
        let mut receiver_type = self.place_type(object);

        while let Type::Reference { inner, .. } = receiver_type {
            receiver_type = *inner;
        }

        let mut argument_types = vec![receiver_type.clone()];
        argument_types.extend(
            current_arguments
                .iter()
                .map(|argument| self.place_type(argument)),
        );
        let builtin = BuiltinCall::resolve(method, &argument_types, None);

        if matches!(
            builtin.as_ref().map(|call| call.kind),
            Some(BuiltinKind::Slice(SliceOperation::Len))
        ) && current_arguments.is_empty()
        {
            return self
                .check_expression(object)
                .map(|_| ReferenceValue::default());
        }

        if matches!(
            builtin.as_ref().map(|call| call.kind),
            Some(BuiltinKind::Pointer(PointerOperation::Offset))
        ) && current_arguments.len() == 1
        {
            let value = self.check_value_expression(object)?;

            for offset in current_arguments {
                self.check_expression(offset)?;
            }

            return Ok(value);
        }

        if let Some(call) = builtin
            && matches!(
                call.kind,
                x_ast::intrinsics::BuiltinKind::Slice(_)
                    | x_ast::intrinsics::BuiltinKind::StringView
                    | x_ast::intrinsics::BuiltinKind::StringBytes
            )
        {
            let mutable = matches!(call.kind, x_ast::intrinsics::BuiltinKind::Slice(operation)
                if operation.receiver_access() == x_ast::intrinsics::SliceReceiverAccess::Mutable);

            return self.with_scope(|this| {
                this.check_expression(object)?;
                if let Some((owner, field)) = this.resolve_lvalue_owner_expression(object) {
                    if this.is_safe_reference_value(object) {
                        this.create_reborrow(&owner, field.as_deref(), mutable)?;
                    } else {
                        this.create_borrow(&owner, field.as_deref(), mutable)?;
                    }
                }

                for current_argument in current_arguments {
                    this.check_transferred_value(current_argument)?;
                }

                Ok(ReferenceValue::default())
            });
        }

        let owner = match receiver_type {
            Type::Custom(name) | Type::GenericInstance { name, .. } => name,
            _ => {
                return Err(BorrowCheckError::UndeclaredVariable(format!(
                    "{receiver_type}::{method}"
                )));
            }
        };

        let (callee, receiver) = self
            .context
            .method_receivers
            .get(&(owner.clone(), method.to_string()))
            .cloned()
            .ok_or_else(|| BorrowCheckError::UndeclaredVariable(format!("{owner}::{method}")))?;
        let receiver_argument = if let Type::Reference {
            is_mut,
            is_unique,
            lifetime,
            ..
        } = receiver
        {
            if matches!(self.place_type(object), Type::Reference { .. }) {
                object.clone()
            } else {
                Expression {
                    span: object.span,
                    value_type: Type::Reference {
                        lifetime,
                        is_mut,
                        is_unique,
                        inner: Box::new(self.place_type(object)),
                    },
                    kind: ExpressionKind::AddressOf {
                        expression: Box::new(object.clone()),
                        is_mut,
                        is_unique,
                    },
                }
            }
        } else {
            object.clone()
        };

        let arguments = std::iter::once(receiver_argument)
            .chain(current_arguments.iter().cloned())
            .collect::<Vec<_>>();

        self.check_call_value(&callee, &arguments)
    }
}
