use super::*;

pub(super) fn flat_validate_unary_expression(
    offset: usize,
    children: &[FlatChild],
) -> Result<(), CstValidationError> {
    let first = flat_require_child(
        FlatNodeContext::new(Rule::unary_expression, offset, children),
        0,
        CstMessage::UnaryOperand.as_str(),
    )?;

    match first.rule {
        Rule::neg_operator
        | Rule::not_operator
        | Rule::bit_not_operator
        | Rule::pre_inc_operator
        | Rule::pre_dec_operator => flat_require_rules(
            FlatNodeContext::new(Rule::unary_expression, offset, children),
            &[first.rule, Rule::unary_expression],
            CstMessage::PrefixParts.as_str(),
        ),
        Rule::address_of | Rule::deref | Rule::postfix_expression if children.len() == 1 => Ok(()),
        _ => Err(CstValidationError::UnexpectedChild {
            offset: first.offset,
            parent: Rule::unary_expression,
            expected: CstMessage::UnaryParts.as_str(),
            actual: first.rule,
        }),
    }
}

pub(super) fn flat_validate_postfix(
    offset: usize,
    children: &[FlatChild],
) -> Result<(), CstValidationError> {
    let child = flat_require_child(
        FlatNodeContext::new(Rule::postfix, offset, children),
        0,
        CstMessage::PostfixOperation.as_str(),
    )?;

    if matches!(
        child.rule,
        Rule::array_access
            | Rule::call_postfix
            | Rule::method_call
            | Rule::try_postfix
            | Rule::field_access
            | Rule::type_cast
    ) && children.len() == 1
    {
        Ok(())
    } else {
        Err(CstValidationError::UnexpectedChild {
            offset: child.offset,
            parent: Rule::postfix,
            expected: CstMessage::PostfixParts.as_str(),
            actual: child.rule,
        })
    }
}

pub(super) fn flat_validate_match_arm(
    offset: usize,
    children: &[FlatChild],
) -> Result<(), CstValidationError> {
    flat_require_first_rule(
        FlatNodeContext::new(Rule::match_arm, offset, children),
        Rule::pattern,
        CstMessage::Pattern.as_str(),
    )?;
    let body = flat_require_child(
        FlatNodeContext::new(Rule::match_arm, offset, children),
        1,
        CstMessage::MatchArmBody.as_str(),
    )?;

    if matches!(
        body.rule,
        Rule::block | Rule::match_return | Rule::expression
    ) {
        Ok(())
    } else {
        Err(CstValidationError::UnexpectedChild {
            offset: body.offset,
            parent: Rule::match_arm,
            expected: CstMessage::BlockReturnExpressionOrExpression.as_str(),
            actual: body.rule,
        })
    }
}
