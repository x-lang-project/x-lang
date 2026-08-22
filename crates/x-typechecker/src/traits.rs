mod resolution;

mod identity;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::iter;

use crate::errors;
use crate::{TypeChecker, helpers::split_top_level_commas};
use identity::*;
use x_ast::SourceError;
use x_ast::TraitDefinition;
use x_ast::{Statement, Type};
use x_ast::{substitute_self_in_statement, substitute_self_in_type};
use x_diagnostics::frontend_messages::FrontendMessage;

mod defaults;

#[derive(PartialEq, Eq)]
struct ResolvedTraitInstance {
    key: String,
    arguments: BTreeMap<String, Type>,
}

impl TypeChecker {
    pub fn validate_trait_impls(
        &mut self,
        statements: &[Statement],
    ) -> Result<(), errors::TypeError> {
        for statement in statements {
            self.validate_trait_impl(statement, statements)
                .map_err(|error| {
                    error.at_source(
                        statement
                            .declaration_key()
                            .and_then(|key| self.declaration_sources.get(&key).copied()),
                    )
                })?;
        }

        Ok(())
    }

    fn validate_trait_impl(
        &mut self,
        statement: &Statement,
        statements: &[Statement],
    ) -> Result<(), errors::TypeError> {
        let Statement::ImplDeclaration(impl_definition) = statement else {
            return Ok(());
        };

        if impl_definition.trait_name.is_empty() {
            return Ok(());
        }

        let trait_key = self.resolve_trait_key(&impl_definition.trait_name)?;
        let Some(trait_key) = trait_key else {
            return Err(errors::TypeError::lookup(FrontendMessage::UnknownTrait {
                trait_name: impl_definition.trait_name.to_string(),
            }));
        };

        let Some(trait_definition) = self.symbols.trait_ast_definitions.get(&trait_key).cloned()
        else {
            return Ok(());
        };

        let trait_type_map = self.bind_trait_type_arguments(
            &trait_definition,
            &impl_definition.trait_name,
            &impl_definition.type_name,
        )?;
        let mut associated_types = self.supertrait_associated_types(
            &trait_definition,
            &impl_definition.type_name,
            &trait_type_map,
            &mut resolution::TraitHierarchy::new(statements),
        )?;
        associated_types.extend(impl_definition.associated_types.clone());
        let required_associated_types = trait_definition
            .associated_types
            .iter()
            .map(|(name, default_type)| {
                impl_definition
                    .associated_types
                    .get(name)
                    .cloned()
                    .or_else(|| {
                        (!matches!(default_type, Type::Unknown)).then(|| default_type.clone())
                    })
                    .map(|value_type| (name.clone(), value_type))
                    .ok_or_else(|| {
                        errors::TypeError::lookup(FrontendMessage::MissingAssociatedType {
                            trait_name: impl_definition.trait_name.to_string(),
                            type_name: impl_definition.type_name.to_string(),
                            name: name.to_string(),
                        })
                    })
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        associated_types.extend(required_associated_types);

        for (method_name, trait_method) in &trait_definition.methods {
            let Some(Statement::Function {
                parameters: trait_parameters,
                return_type: trait_return,
                body: trait_body,
                ..
            }) = trait_method
            else {
                continue;
            };

            let impl_method = impl_definition.methods.iter().find(
                |method| matches!(method, Statement::Function { name, .. } if name == method_name),
            );

            if impl_method.is_none() && trait_body.is_none() {
                return Err(errors::TypeError::lookup(
                    FrontendMessage::MissingTraitMethod {
                        trait_name: impl_definition.trait_name.to_string(),
                        type_name: impl_definition.type_name.to_string(),
                        method_name: method_name.to_string(),
                    },
                ));
            }

            let Some(Statement::Function {
                parameters: impl_parameters,
                return_type: impl_return,
                ..
            }) = impl_method
            else {
                continue;
            };

            let mut expected_parameters = trait_parameters.clone();
            let mut expected_return = trait_return.clone();
            let mut actual_parameters = impl_parameters.clone();
            let mut actual_return = impl_return.clone();

            for value_type in expected_parameters
                .iter_mut()
                .map(|parameter| &mut parameter.value_type)
                .chain(iter::once(&mut expected_return))
            {
                substitute_associated_self_types(value_type, &associated_types);
                substitute_self_in_type(value_type, &impl_definition.type_name);
                *value_type = self.substitute_type(value_type, &trait_type_map);
            }

            for value_type in actual_parameters
                .iter_mut()
                .map(|parameter| &mut parameter.value_type)
                .chain(iter::once(&mut actual_return))
            {
                substitute_associated_self_types(value_type, &associated_types);
                substitute_self_in_type(value_type, &impl_definition.type_name);
            }

            if expected_parameters.len() != actual_parameters.len() {
                return Err(errors::TypeError::ArityMismatch {
                    expected: expected_parameters.len(),
                    got: actual_parameters.len(),
                    symbol_name: format!("{}::{method_name}", impl_definition.trait_name),
                });
            }

            for (expected, got) in expected_parameters.iter().zip(actual_parameters.iter()) {
                let expected_type = self.resolve_type(&expected.value_type)?;
                let got_type = self.resolve_type(&got.value_type)?;

                if !self.storage_types_compatible(&expected_type, &got_type)
                    || !self.storage_types_compatible(&got_type, &expected_type)
                {
                    return Err(crate::type_mismatch!(
                        format!(
                            "{expected_type} for trait method `{method_name}` parameter `{}`",
                            expected.name
                        ),
                        format!("{got_type}")
                    ));
                }
            }

            let expected_return = self.resolve_type(&expected_return)?;
            let got_return = self.resolve_type(&actual_return)?;

            if !self.storage_types_compatible(&expected_return, &got_return)
                || !self.storage_types_compatible(&got_return, &expected_return)
            {
                return Err(crate::type_mismatch!(
                    format!("{expected_return} as return type of trait method `{method_name}`"),
                    format!("{got_return}")
                ));
            }
        }

        Ok(())
    }
}
