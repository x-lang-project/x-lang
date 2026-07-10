use std::collections::BTreeMap;

use pest::iterators::Pair;

use super::generics::{LifetimeSpelling, parse_generic_parameters};
use crate::{
    Rule,
    function::parse_function_definition,
    lowering_error::{LoweringError, LoweringResult},
    types::parse_type,
};
use x_ast::{ImplDefinition, Statement, TraitDefinition, Type};
use x_diagnostics::parser_messages::{ExpectedSyntax, ParserIssue};

pub(super) fn parse_type_alias_declaration(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let mut name = None;
    let mut generic_parameters = Vec::new();
    let mut value_type = None;

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::identifier => {
                name = Some(child.as_str().to_string());
            }
            Rule::generic_parameter_definition_list => {
                generic_parameters = parse_generic_parameters(child, LifetimeSpelling::Quoted)?;
            }
            Rule::type_specifier | Rule::r#type => {
                value_type = Some(parse_type(child)?);
            }
            _ => {}
        }
    }

    Ok(Statement::TypeAlias {
        name: name.ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::AliasName))?,
        generic_parameters,
        value_type: value_type
            .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::AliasTargetType))?,
    })
}

pub fn parse_trait_declaration(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let mut name: Option<String> = None;
    let mut generic_parameters = Vec::new();
    let mut associated_types = BTreeMap::new();
    let mut methods = BTreeMap::new();
    let mut supertraits = Vec::new();

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::identifier => {
                name = Some(child.as_str().to_string());
            }

            Rule::generic_parameter_definition_list => {
                generic_parameters = parse_generic_parameters(child, LifetimeSpelling::Quoted)?;
            }
            Rule::generic_bound_list => {
                supertraits = crate::types::parse_generic_bound_list(child)?;
            }
            Rule::trait_item => {
                let mut item_inner = child.clone().into_inner();
                let Some(item_pair) = item_inner.next() else {
                    return Err(LoweringError::missing(&child, ExpectedSyntax::TraitItem));
                };

                if item_pair.as_rule() == Rule::associated_type_declaration {
                    let (name, value_type) = parse_associated_type_declaration(item_pair)?;
                    associated_types.insert(name, value_type);
                    continue;
                }

                let function_statement = parse_function_definition(item_pair)?;

                let method_name = match &function_statement {
                    Statement::Function { name, .. } => name.clone(),
                    _ => {
                        return Err(LoweringError::invalid(
                            &error_pair,
                            ParserIssue::InvalidTraitMethodLowering,
                        ));
                    }
                };

                methods.insert(method_name, Some(function_statement));
            }

            _ => {}
        }
    }

    Ok(Statement::TraitDeclaration(TraitDefinition {
        supertraits,
        name: name.ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::TraitName))?,
        generic_parameters,
        associated_types,
        methods,
    }))
}

fn parse_associated_type_declaration(pair: Pair<Rule>) -> LoweringResult<(String, Type)> {
    let error_pair = pair.clone();
    let mut name: Option<String> = None;
    let mut value_type: Option<Type> = None;

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::identifier => {
                name = Some(child.as_str().to_string());
            }
            Rule::type_specifier | Rule::r#type => {
                value_type = Some(parse_type(child)?);
            }
            _ => {}
        }
    }

    Ok((
        name.ok_or_else(|| {
            LoweringError::missing(&error_pair, ExpectedSyntax::AssociatedTypeName)
        })?,
        value_type.unwrap_or(Type::Unknown),
    ))
}

pub fn parse_impl_definition(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::ImplDeclaration))?;

    match inner.as_rule() {
        Rule::trait_impl_declaration | Rule::inherent_impl_declaration => {
            parse_impl_declaration(inner)
        }
        _ => Err(LoweringError::invalid(
            &inner,
            ParserIssue::InvalidImplDeclaration,
        )),
    }
}

fn parse_impl_declaration(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let is_trait_impl = pair.as_rule() == Rule::trait_impl_declaration;
    let error_pair = pair.clone();
    let mut trait_name: Option<String> = None;
    let mut type_name: Option<Type> = None;
    let mut associated_types = BTreeMap::new();
    let mut methods = Vec::new();
    let mut generic_parameters = Vec::new();

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::trait_path | Rule::custom_type | Rule::identifier if is_trait_impl => {
                if trait_name.is_none() {
                    parse_type(child.clone())?;
                    trait_name = Some(child.as_str().trim().to_string());
                }
            }

            Rule::type_specifier | Rule::r#type => {
                if type_name.is_none() {
                    type_name = Some(parse_type(child)?);
                }
            }

            Rule::generic_parameter_definition_list => {
                generic_parameters = parse_generic_parameters(child, LifetimeSpelling::Quoted)?;
            }
            Rule::impl_item => {
                let item_error = child.clone();
                let item_pair = child
                    .into_inner()
                    .next()
                    .ok_or_else(|| LoweringError::missing(&item_error, ExpectedSyntax::ImplItem))?;

                match item_pair.as_rule() {
                    Rule::associated_type_declaration => {
                        let (name, value_type) = parse_associated_type_declaration(item_pair)?;
                        associated_types.insert(name, value_type);
                    }
                    Rule::function_definition => {
                        methods.push(parse_function_definition(item_pair)?)
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }

    Ok(Statement::ImplDeclaration(ImplDefinition {
        trait_name: if is_trait_impl {
            trait_name
                .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::TraitName))?
        } else {
            String::new()
        },
        type_name: type_name.ok_or_else(|| {
            LoweringError::missing(
                &error_pair,
                if is_trait_impl {
                    ExpectedSyntax::TraitImplTargetType
                } else {
                    ExpectedSyntax::ImplTargetType
                },
            )
        })?,
        generic_parameters,
        associated_types,
        methods,
    }))
}
