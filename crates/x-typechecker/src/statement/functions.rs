use super::*;

impl TypeChecker {
    pub(super) fn check_impl_statement(
        &mut self,
        impl_definition: &mut x_ast::ImplDefinition,
    ) -> Result<(), TypeError> {
        let owner_name = match &impl_definition.type_name {
            Type::Custom(name) => name.clone(),
            Type::GenericInstance { .. } => format!("{}", impl_definition.type_name),
            Type::TypeParameter(name) => name.clone(),
            other => other.to_string(),
        };

        for method in &mut impl_definition.methods {
            let Statement::Function {
                name,
                parameters,
                body,
                generic_parameters,
                return_type,
                is_pure,
                is_throws,
                is_static,
                ..
            } = method
            else {
                continue;
            };

            if generic_parameters
                .iter()
                .any(|parameter| !matches!(parameter, x_ast::GenericParameter::Lifetime { .. }))
            {
                return Err(TypeError::ExecutableType {
                    value_type: Type::Custom(format!("{owner_name}::{name}")),
                    reason: ExecutableTypeReason::GenericMethodBody,
                });
            }

            let mut self_substitution = HashMap::new();
            self_substitution.insert("Self".to_string(), impl_definition.type_name.clone());
            for parameter in parameters.iter_mut() {
                parameter.value_type =
                    self.substitute_type(&parameter.value_type, &self_substitution);
            }
            *return_type = self.substitute_type(return_type, &self_substitution);

            let qualified_name = format!("{owner_name}::{name}");

            if generic_parameters
                .iter()
                .any(|parameter| !matches!(parameter, x_ast::GenericParameter::Lifetime { .. }))
                || parameters
                    .iter()
                    .any(|parameter| Self::type_contains_placeholder(&parameter.value_type))
                || Self::type_contains_placeholder(return_type)
            {
                continue;
            }

            self.symbols.add_function(
                qualified_name.clone(),
                crate::typed_function_signature!(
                    parameters,
                    return_type,
                    *is_pure,
                    *is_throws,
                    *is_static,
                ),
            );

            self.check_function_body(
                qualified_name,
                *is_pure,
                parameters,
                body.as_deref_mut()
                    .map(Vec::as_mut_slice)
                    .unwrap_or_default(),
            )?;
        }

        Ok(())
    }
}

impl TypeChecker {
    pub(super) fn check_function_body(
        &mut self,
        name: String,
        is_pure: bool,
        parameters: &[Parameter],
        body: &mut [Statement],
    ) -> Result<(), TypeError> {
        self.with_named_function(name, is_pure, parameters, |checker| {
            for parameter in parameters {
                checker
                    .symbols
                    .add_variable(parameter.name.clone(), parameter.value_type.clone());
            }

            body.iter_mut()
                .try_for_each(|statement| checker.check_statement(statement))
        })
    }

    pub(super) fn type_contains_placeholder(value_type: &Type) -> bool {
        value_type.any_nested_type(|inner| matches!(inner, Type::TypeParameter(_)))
    }
}

impl TypeChecker {
    pub(super) fn check_scoped_statements(
        &mut self,
        statements: &mut [Statement],
    ) -> Result<(), TypeError> {
        self.with_lexical_scope(|checker| {
            statements
                .iter_mut()
                .try_for_each(|statement| checker.check_statement(statement))
        })
    }
}
