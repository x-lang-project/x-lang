use super::*;

impl TypeChecker {
    pub(super) fn check_become_statement(
        &mut self,
        call: &mut Expression,
    ) -> Result<(), TypeError> {
        // Tail calls forward the error channel to their caller.
        let result_type =
            self.with_handled_call_operand(|checker| checker.check_expression(call))?;
        let ExpressionKind::FunctionCall { target: callee, .. } = &call.kind else {
            return Err(TypeError::inference(FrontendMessage::IndirectBecomeCall));
        };

        let callee = callee.symbol_name();
        let caller = self
            .current_function_name()
            .cloned()
            .ok_or_else(|| TypeError::inference(FrontendMessage::BecomeOutsideFunction))?;
        let caller_signature = self.symbols.get_function(&caller).cloned().ok_or_else(|| {
            TypeError::inference(FrontendMessage::MissingBecomeCallerAbi {
                caller: caller.to_string(),
            })
        })?;
        let callee_signature = self.symbols.get_function(&callee).cloned().ok_or_else(|| {
            TypeError::inference(FrontendMessage::MissingBecomeCalleeAbi {
                callee: callee.to_string(),
            })
        })?;

        self.unify(&caller_signature.return_type, &result_type)?;
        let caller_parameters = caller_signature
            .parameter_types
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                !self
                    .symbols
                    .function_static_parameters
                    .get(&caller)
                    .and_then(|flags| flags.get(*index))
                    .copied()
                    .unwrap_or(false)
            })
            .map(|(_, value_type)| self.resolve(value_type))
            .collect::<Vec<_>>();
        let callee_parameters = callee_signature
            .parameter_types
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                !self
                    .symbols
                    .function_static_parameters
                    .get(callee.as_ref())
                    .and_then(|flags| flags.get(*index))
                    .copied()
                    .unwrap_or(false)
            })
            .map(|(_, value_type)| self.resolve(value_type))
            .collect::<Vec<_>>();

        if caller_parameters != callee_parameters
            || caller_signature.is_throws != callee_signature.is_throws
            || self.resolve(&caller_signature.return_type)
                != self.resolve(&callee_signature.return_type)
        {
            return Err(TypeError::inference(FrontendMessage::BecomeAbiMismatch {
                callee: callee.to_string(),
                caller: caller.to_string(),
                caller_parameters: format!("{:?}", caller_parameters),
                caller_return_type: caller_signature.return_type.to_string(),
                callee_parameters: format!("{:?}", callee_parameters),
                callee_return_type: callee_signature.return_type.to_string(),
            }));
        }

        Ok(())
    }
}

impl TypeChecker {
    pub(super) fn check_return_statement(
        &mut self,
        value: Option<&mut Expression>,
    ) -> Result<(), TypeError> {
        let mut value = value;

        if let Some(expression) = value.as_deref_mut()
            && matches!(expression.kind, ExpressionKind::StructInstantiate(_))
        {
            let expected = self.return_types.last().cloned().or_else(|| {
                self.current_function_name()
                    .and_then(|name| self.symbols.get_function(name))
                    .map(|signature| signature.return_type.clone())
            });

            if let Some(expected) = expected {
                expression.value_type = self.resolve_type(&expected)?;
            }
        }

        let expected = self.return_types.last().cloned().or_else(|| {
            self.current_function_name()
                .and_then(|name| self.symbols.get_function(name))
                .map(|signature| signature.return_type.clone())
        });
        let current_return_type = if let Some(value_expression) = value {
            let type_value = self.check_expression(value_expression)?;

            if crate::expression::is_integer_literal(value_expression)
                && let Some(expected) = &expected
                && expected.is_integer()
            {
                value_expression.value_type = expected.clone();

                expected.clone()
            } else {
                value_expression.value_type = type_value.clone();

                type_value
            }
        } else {
            Type::Void
        };

        if let Some(expected) = self.return_types.last().cloned() {
            self.unify(&expected, &current_return_type)?;
        } else if let Some(function_name) = self.current_function_name().cloned()
            && let Some(current_signature) = self.symbols.get_function(&function_name).cloned()
        {
            self.unify(&current_signature.return_type, &current_return_type)
                .map_err(|error| {
                    error.map_expected(|expected| {
                        format!("{expected} as return type of '{function_name}'")
                    })
                })?;
        }

        Ok(())
    }
}
