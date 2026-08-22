use super::*;

impl TypeChecker {
    fn register_match_bindings(&mut self, bindings: &[String], elements: &[Type]) {
        for (binding, value_type) in bindings.iter().zip(elements) {
            if binding != "_" {
                self.symbols
                    .add_variable(binding.clone(), value_type.clone());
            }
        }
    }

    pub(super) fn check_match(
        &mut self,
        scrutinee: &mut Box<Expression>,
        arms: &mut Vec<MatchArm>,
    ) -> Result<Type, TypeError> {
        let scrutinee_type = self.check_expression(scrutinee)?;
        let scrutinee_type = self.resolve(&scrutinee_type);

        let mut result_type: Option<Type> = None;

        for arm in arms {
            let arm_type = self.check_match_arm(arm, &scrutinee_type)?;

            if let Some(existing) = &result_type {
                self.unify(existing, &arm_type)?;

                if *existing == Type::Never {
                    result_type = Some(arm_type);
                }
            } else {
                result_type = Some(arm_type);
            }
        }

        Ok(result_type.unwrap_or(Type::Void))
    }

    fn check_match_arm(
        &mut self,
        arm: &mut MatchArm,
        scrutinee_type: &Type,
    ) -> Result<Type, TypeError> {
        self.with_lexical_scope(|checker| {
            match &arm.pattern {
                Pattern::Tuple(bindings) => {
                    let Type::Tuple(items) = scrutinee_type else {
                        return Err(TypeError::inference(FrontendMessage::NonTuplePattern));
                    };

                    if items.len() != bindings.len() {
                        return Err(TypeError::inference(
                            FrontendMessage::TuplePatternArityMismatch,
                        ));
                    }

                    checker.register_match_bindings(bindings, items);
                }
                Pattern::Enum { .. } => {
                    return Err(TypeError::ExecutableType {
                        value_type: scrutinee_type.clone(),
                        reason: ExecutableTypeReason::MissingEnumDiscriminantAbi,
                    });
                }
                Pattern::Identifier(name) => {
                    checker
                        .symbols
                        .add_variable(name.clone(), scrutinee_type.clone());
                }
                _ => {}
            }

            let body_type = checker.check_block_expression_type(&mut arm.body)?;
            let arm_type = if let Some(result) = &mut arm.result {
                let result_type = checker.check_expression(result)?;

                if body_type == Type::Never {
                    Type::Never
                } else {
                    result_type
                }
            } else {
                body_type
            };

            Ok(arm_type)
        })
    }

    pub(super) fn check_block_expression_type(
        &mut self,
        statements: &mut [Statement],
    ) -> Result<Type, TypeError> {
        let mut result = Type::Void;
        let mut diverges = false;

        for statement in statements {
            match statement {
                Statement::Expression { expression } => {
                    result = self.check_expression(expression)?;
                }
                Statement::Return { .. } => {
                    self.check_statement(statement)?;
                    diverges = true;
                }
                Statement::Comment(_) => self.check_statement(statement)?,
                other => {
                    self.check_statement(other)?;
                    result = Type::Void;
                }
            }
        }

        Ok(if diverges { Type::Never } else { result })
    }
}

impl TypeChecker {
    pub(super) fn check_if_expression(
        &mut self,
        condition: &mut Expression,
        then_block: &mut [Statement],
        else_block: Option<&mut [Statement]>,
    ) -> Result<Type, TypeError> {
        let condition_type = self.check_expression(condition)?;
        self.unify(&Type::Boolean, &condition_type)?;

        let then_type =
            self.with_lexical_scope(|checker| checker.check_block_expression_type(then_block))?;

        let else_type = if let Some(else_block) = else_block {
            self.with_lexical_scope(|checker| checker.check_block_expression_type(else_block))?
        } else {
            Type::Void
        };

        self.unify(&then_type, &else_type)?;

        Ok(self.resolve(if then_type == Type::Never {
            &else_type
        } else {
            &then_type
        }))
    }
}
