use pest::iterators::{Pair, Pairs};

use super::contracts::{
    flat_require_all_rules, flat_require_any_rule, flat_require_fields, flat_require_first_of,
    flat_require_first_rule, flat_require_remaining_rules, flat_require_rules,
    flat_require_single_of,
};
use super::{CstValidationError, FlatChild, FlatNodeContext, Rule};
use x_diagnostics::parser_messages::CstMessage;

pub(super) fn validate(
    rule: Rule,
    offset: usize,
    children: &[FlatChild],
) -> Option<Result<(), CstValidationError>> {
    let context = FlatNodeContext::new(rule, offset, children);
    let result = match rule {
        Rule::function_definition => flat_require_fields(
            context,
            &[
                (&[Rule::identifier], CstMessage::FunctionName),
                (&[Rule::function_parameter_list], CstMessage::ParameterList),
            ],
        ),
        Rule::function_parameter_list => match children.first() {
            Some(child) if child.rule != Rule::parameter_list_nonempty => {
                Err(CstValidationError::UnexpectedChild {
                    offset: child.offset,
                    parent: rule,
                    expected: CstMessage::ParameterList.as_str(),
                    actual: child.rule,
                })
            }
            _ => Ok(()),
        },
        Rule::parameter_list_nonempty => {
            flat_require_all_rules(context, Rule::parameter_item, "parameters")
        }
        Rule::parameter_item => flat_require_first_of(
            context,
            &[
                Rule::parameter_self,
                Rule::parameter_owned_self,
                Rule::parameter_normal,
            ],
            CstMessage::SelfOrNamedParameter.as_str(),
        ),
        Rule::parameter_normal => flat_require_any_rule(
            context,
            Rule::identifier,
            CstMessage::ParameterName.as_str(),
        ),
        Rule::struct_field => flat_require_fields(
            context,
            &[
                (&[Rule::identifier], CstMessage::FieldName),
                (&[Rule::type_specifier, Rule::r#type], CstMessage::FieldType),
            ],
        ),
        Rule::struct_instantiate => {
            flat_require_first_rule(context, Rule::pattern_path, CstMessage::StructName.as_str())
        }
        Rule::struct_initialise_fields => flat_require_all_rules(
            context,
            Rule::struct_initialise_field,
            CstMessage::FieldInitializers.as_str(),
        ),
        Rule::struct_initialise_field => {
            flat_require_first_rule(context, Rule::identifier, CstMessage::FieldName.as_str())
        }
        Rule::struct_declaration => {
            flat_require_any_rule(context, Rule::identifier, CstMessage::TypeName.as_str())
        }
        Rule::type_alias_declaration => flat_require_fields(
            context,
            &[
                (&[Rule::identifier], CstMessage::AliasName),
                (&[Rule::type_specifier], CstMessage::AliasTargetType),
            ],
        ),
        Rule::trait_declaration => {
            flat_require_any_rule(context, Rule::identifier, CstMessage::TraitName.as_str())
        }
        Rule::associated_type_declaration => flat_require_any_rule(
            context,
            Rule::identifier,
            CstMessage::AssociatedTypeName.as_str(),
        ),
        Rule::impl_definition => flat_require_single_of(
            context,
            &[
                Rule::trait_impl_declaration,
                Rule::inherent_impl_declaration,
            ],
            CstMessage::TraitOrInherentImplDeclaration.as_str(),
        ),
        Rule::trait_impl_declaration => flat_require_fields(
            context,
            &[
                (&[Rule::trait_path], CstMessage::TraitPath),
                (
                    &[Rule::type_specifier],
                    CstMessage::ImplementationTargetType,
                ),
            ],
        ),
        Rule::inherent_impl_declaration => flat_require_any_rule(
            context,
            Rule::type_specifier,
            CstMessage::ImplementationTargetType.as_str(),
        ),
        Rule::variable_declaration => flat_require_fields(
            context,
            &[
                (&[Rule::let_pattern], CstMessage::BindingPattern),
                (
                    &[Rule::r#type, Rule::expression],
                    CstMessage::BindingTypeOrValue,
                ),
            ],
        ),
        Rule::assignment => flat_require_rules(
            context,
            &[Rule::l_value, Rule::assign_operator, Rule::expression],
            CstMessage::AssignmentParts.as_str(),
        ),
        Rule::import_statement => flat_require_first_rule(
            context,
            Rule::import_path,
            CstMessage::NonEmptyImportPath.as_str(),
        ),
        Rule::external_block => flat_validate_external_block(offset, children),
        Rule::external_block_function => flat_require_fields(
            context,
            &[
                (&[Rule::identifier], CstMessage::ExternalFunctionName),
                (
                    &[Rule::function_parameter_list],
                    CstMessage::ExternalFunctionParameterList,
                ),
            ],
        ),
        _ => return None,
    };

    Some(result)
}

fn flat_validate_external_block(
    offset: usize,
    children: &[FlatChild],
) -> Result<(), CstValidationError> {
    let first_item = children
        .iter()
        .position(|child| child.rule == Rule::external_block_function)
        .unwrap_or(children.len());
    let prefix = children.get(..first_item).unwrap_or_default();

    if prefix
        .iter()
        .filter(|child| child.rule != Rule::attribute)
        .count()
        > 1
    {
        return Err(CstValidationError::UnsupportedShape {
            offset,
            rule: Rule::external_block,
            detail: CstMessage::DuplicateExternBlockAbi.as_str(),
        });
    }

    flat_require_remaining_rules(
        FlatNodeContext::new(Rule::external_block, offset, children),
        first_item,
        &[Rule::external_block_function],
        CstMessage::ExternalFunctionDeclarations.as_str(),
    )
}

pub(super) fn validate_external_block_function(
    offset: usize,
    mut children: Pairs<Rule>,
) -> Result<(), CstValidationError> {
    let parameters = children
        .find(|child| child.as_rule() == Rule::function_parameter_list)
        .ok_or(CstValidationError::MissingChild {
            offset,
            parent: Rule::external_block_function,
            expected: CstMessage::ExternalFunctionParameterList.as_str(),
        })?;

    for item in parameters.into_inner().flat_map(Pair::into_inner) {
        let Some(parameter) = item.clone().into_inner().next() else {
            return Err(CstValidationError::MissingChild {
                offset: item.as_span().start(),
                parent: Rule::parameter_item,
                expected: CstMessage::ExternalParameter.as_str(),
            });
        };

        if parameter.as_rule() != Rule::parameter_normal
            || !parameter
                .clone()
                .into_inner()
                .any(|part| part.as_rule() == Rule::type_specifier)
        {
            return Err(CstValidationError::UnsupportedShape {
                offset: parameter.as_span().start(),
                rule: Rule::external_block_function,
                detail: CstMessage::UnnamedExternBlockParameter.as_str(),
            });
        }
    }

    Ok(())
}
