mod projections;
use super::contracts::{
    flat_require_any_of, flat_require_any_rule, flat_require_child, flat_require_first_rule,
    flat_require_rules, flat_require_single_of, flat_require_suffixes,
};
use super::{CstValidationError, FlatChild, FlatNodeContext, Rule};
use projections::*;
use x_diagnostics::parser_messages::CstMessage;

pub(super) fn validate(
    rule: Rule,
    offset: usize,
    children: &[FlatChild],
) -> Option<Result<(), CstValidationError>> {
    let context = FlatNodeContext::new(rule, offset, children);
    let result = match rule {
        Rule::expression
        | Rule::type_specifier
        | Rule::type_atom
        | Rule::return_type_annotation => {
            flat_require_child(context, 0, CstMessage::InnerNode.as_str()).map(|_| ())
        }
        Rule::ternary_expression => {
            if matches!(children.len(), 1 | 3) {
                Ok(())
            } else {
                Err(CstValidationError::UnsupportedShape {
                    offset,
                    rule,
                    detail: CstMessage::InvalidTernaryArity.as_str(),
                })
            }
        }
        Rule::cast_expression => flat_require_suffixes(
            context,
            (Rule::unary_expression, CstMessage::UnaryExpression),
            (Rule::type_cast, CstMessage::TypeCasts),
        ),
        Rule::unary_expression => flat_validate_unary_expression(offset, children),
        Rule::address_of | Rule::deref => flat_require_any_rule(
            context,
            Rule::unary_expression,
            CstMessage::TargetUnaryExpression.as_str(),
        ),
        Rule::postfix_expression => flat_require_suffixes(
            context,
            (Rule::primary, CstMessage::PrimaryExpression),
            (Rule::postfix, CstMessage::PostfixOperations),
        ),
        Rule::postfix => flat_validate_postfix(offset, children),
        Rule::array_access | Rule::type_cast => {
            flat_require_child(context, 0, CstMessage::Operand.as_str()).map(|_| ())
        }
        Rule::field_access | Rule::method_call => {
            flat_require_any_rule(context, Rule::identifier, CstMessage::Identifier.as_str())
        }
        Rule::primary => {
            flat_require_child(context, 0, CstMessage::PrimaryExpression.as_str()).map(|_| ())
        }
        Rule::match_expression => flat_require_suffixes(
            context,
            (Rule::expression, CstMessage::ScrutineeExpression),
            (Rule::match_arm, CstMessage::MatchArms),
        ),
        Rule::match_arm => flat_validate_match_arm(offset, children),
        Rule::match_return => flat_require_first_rule(
            context,
            Rule::expression,
            CstMessage::ReturnExpression.as_str(),
        ),
        Rule::enum_pattern => flat_require_first_rule(
            context,
            Rule::pattern_path,
            CstMessage::EnumVariant.as_str(),
        ),
        Rule::r#type | Rule::array_type | Rule::reference_type | Rule::pointer_type => {
            flat_require_type_base(rule, offset, children)
        }
        Rule::function_call => {
            flat_require_first_rule(context, Rule::path, CstMessage::CallablePath.as_str())
        }
        Rule::l_value => flat_require_single_of(
            context,
            &[Rule::address_of, Rule::deref, Rule::primary_lvalue],
            CstMessage::LvalueParts.as_str(),
        ),
        Rule::primary_lvalue => flat_require_single_of(
            context,
            &[Rule::postfix_expression, Rule::l_value],
            CstMessage::PrimaryLvalueParts.as_str(),
        ),
        Rule::external_function_declaration => flat_require_any_rule(
            context,
            Rule::identifier,
            CstMessage::ExternalFunctionName.as_str(),
        ),
        Rule::external_parameter => flat_require_first_rule(
            context,
            Rule::identifier,
            CstMessage::ParameterName.as_str(),
        ),
        _ => return None,
    };

    Some(result)
}

pub(super) fn flat_validate_array_repetition(
    offset: usize,
    children: &[FlatChild],
) -> Result<(), CstValidationError> {
    flat_require_first_rule(
        FlatNodeContext::new(Rule::array_literal, offset, children),
        Rule::expression,
        CstMessage::ArrayRepetitionValue.as_str(),
    )?;
    let count = flat_require_child(
        FlatNodeContext::new(Rule::array_literal, offset, children),
        children.len().saturating_sub(1),
        CstMessage::ArrayRepetitionCount.as_str(),
    )?;

    if matches!(count.rule, Rule::integer | Rule::identifier) {
        Ok(())
    } else {
        Err(CstValidationError::UnexpectedChild {
            offset: count.offset,
            parent: Rule::array_literal,
            expected: CstMessage::IntegerOrIdentifierRepetitionCount.as_str(),
            actual: count.rule,
        })
    }
}

pub(super) fn binary_chain_contract(rule: Rule) -> Option<(Rule, &'static [Rule])> {
    match rule {
        Rule::binary_expression => Some((
            Rule::cast_expression,
            &[
                Rule::or_operator,
                Rule::and_operator,
                Rule::bit_or_operator,
                Rule::xor_operator,
                Rule::bit_and_operator,
                Rule::eq_operator,
                Rule::neq_operator,
                Rule::lt_operator,
                Rule::gt_operator,
                Rule::le_operator,
                Rule::ge_operator,
                Rule::shl_operator,
                Rule::shr_operator,
                Rule::add_operator,
                Rule::sub_operator,
                Rule::mul_operator,
                Rule::div_operator,
                Rule::mod_operator,
            ],
        )),

        _ => None,
    }
}

pub(super) fn flat_require_type_base(
    parent: Rule,
    offset: usize,
    children: &[FlatChild],
) -> Result<(), CstValidationError> {
    flat_require_any_of(
        FlatNodeContext::new(parent, offset, children),
        &[
            Rule::reference_type,
            Rule::type_atom,
            Rule::r#type,
            Rule::basic_type,
            Rule::custom_type,
            Rule::array_type,
            Rule::pointer_type,
            Rule::tuple_type,
            Rule::function_type,
        ],
        CstMessage::BaseType.as_str(),
    )
}
