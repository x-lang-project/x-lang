use super::*;

impl TypeChecker {
    pub(super) fn check_anonymous_function(
        &mut self,
        parameters: &[String],
        body: &mut [Statement],
        return_type: &Option<Type>,
    ) -> Result<Type, TypeError> {
        let expected = return_type.clone().unwrap_or_else(|| self.fresh_type());
        let inferred_return = self.with_anonymous_function(|checker| {
            checker.with_return_type(expected.clone(), |checker| {
                checker.with_lexical_scope(|checker| {
                    for parameter in parameters.iter() {
                        if parameter != "_" {
                            checker
                                .symbols
                                .add_variable(parameter.clone(), Type::Unknown);
                        }
                    }

                    checker.check_block_expression_type(body)
                })
            })
        })?;
        self.unify(&expected, &inferred_return)?;
        let return_value = match self.resolve(&expected) {
            Type::Infer(_) => inferred_return,
            resolved => resolved,
        };

        Ok(Type::Function {
            parameters: vec![Type::Unknown; parameters.len()],
            return_value: Box::new(return_value),
        })
    }
}
