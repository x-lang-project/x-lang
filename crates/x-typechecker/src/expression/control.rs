use super::*;
use x_diagnostics::frontend_messages::FrontendMessage;

impl TypeChecker {
    pub(super) fn check_try(&mut self, inner: &mut Expression) -> Result<Type, TypeError> {
        if self.in_anonymous_function() {
            return Err(TypeError::inference(
                FrontendMessage::PropagationOutsideFunction,
            ));
        }

        let enclosing = self
            .current_function_name()
            .cloned()
            .ok_or_else(|| TypeError::inference(FrontendMessage::PropagationOutsideFunction))?;
        let enclosing_signature = self.symbols.get_function(&enclosing).ok_or_else(|| {
            TypeError::inference(FrontendMessage::MissingPropagationCaller {
                enclosing: enclosing.to_string(),
            })
        })?;

        if !enclosing_signature.is_throws {
            return Err(TypeError::inference(
                FrontendMessage::NonThrowingPropagationCaller {
                    enclosing: enclosing.to_string(),
                },
            ));
        }

        let inner_type =
            self.with_handled_call_operand(|checker| checker.check_expression(inner))?;

        let ExpressionKind::FunctionCall { target: name, .. } = &inner.kind else {
            return Err(TypeError::inference(
                FrontendMessage::IndirectPropagationCall,
            ));
        };

        let name = name.symbol_name();
        let called_signature = self.symbols.get_function(&name).ok_or_else(|| {
            TypeError::inference(FrontendMessage::MissingThrowingAbi {
                name: name.to_string(),
            })
        })?;

        if !called_signature.is_throws {
            return Err(TypeError::inference(
                FrontendMessage::NonThrowingPropagationCall {
                    name: name.to_string(),
                },
            ));
        }

        let success_type = self.resolve_type(&inner_type)?;
        inner.value_type = success_type.clone();

        Ok(success_type)
    }
}

impl TypeChecker {
    pub(super) fn check_scoped_block(
        &mut self,
        statements: &mut [Statement],
        expression: Option<&mut Expression>,
    ) -> Result<Type, TypeError> {
        self.with_lexical_scope(|checker| {
            let mut diverges = false;

            for statement in statements {
                checker.check_statement(statement)?;
                diverges |= matches!(
                    statement,
                    Statement::Return { .. } | Statement::Throw { .. } | Statement::Become { .. }
                );
            }

            if diverges {
                return Ok(Type::Never);
            }

            match expression {
                Some(expression) => checker.check_expression(expression),
                None => Ok(Type::Void),
            }
        })
    }
}
