use super::contracts::{flat_require_any_rule, flat_require_child, flat_require_rules};
use super::{CstValidationError, FlatChild, FlatNodeContext, Rule};
use x_diagnostics::parser_messages::CstMessage;

pub(super) fn validate(
    rule: Rule,
    offset: usize,
    children: &[FlatChild],
) -> Option<Result<(), CstValidationError>> {
    let context = FlatNodeContext::new(rule, offset, children);
    let result = match rule {
        Rule::statement => {
            flat_require_child(context, 0, CstMessage::StatementNode.as_str()).map(|_| ())
        }
        Rule::while_loop => flat_require_rules(
            context,
            &[Rule::expression, Rule::block],
            CstMessage::ConditionExpressionAndBlock.as_str(),
        ),
        Rule::for_range_loop => {
            let children = without_flat_parallel_prefix(children);

            flat_require_rules(
                FlatNodeContext::new(rule, offset, children),
                &[
                    Rule::identifier,
                    Rule::expression,
                    Rule::expression,
                    Rule::block,
                ],
                CstMessage::ForRangeParts.as_str(),
            )
        }
        Rule::for_each_loop => {
            let children = without_flat_parallel_prefix(children);

            flat_require_rules(
                FlatNodeContext::new(rule, offset, children),
                &[Rule::identifier, Rule::expression, Rule::block],
                CstMessage::ForEachParts.as_str(),
            )
        }
        Rule::if_statement => flat_validate_if(offset, children),
        Rule::unsafe_block | Rule::loop_statement => {
            flat_require_any_rule(context, Rule::block, CstMessage::Block.as_str())
        }
        Rule::invariant_statement => {
            flat_require_child(context, 0, CstMessage::InvariantBlock.as_str()).map(|_| ())
        }
        Rule::throw_statement => flat_require_any_rule(
            context,
            Rule::expression,
            CstMessage::ErrorExpression.as_str(),
        ),
        _ => return None,
    };

    Some(result)
}

fn flat_validate_if(offset: usize, children: &[FlatChild]) -> Result<(), CstValidationError> {
    flat_require_rules(
        FlatNodeContext::new(Rule::if_statement, offset, children),
        &[Rule::expression, Rule::block],
        CstMessage::ConditionExpressionAndThenBlock.as_str(),
    )?;
    if let Some(otherwise) = children.get(2)
        && !matches!(otherwise.rule, Rule::block | Rule::if_statement)
    {
        return Err(CstValidationError::UnexpectedChild {
            offset: otherwise.offset,
            parent: Rule::if_statement,
            expected: CstMessage::ElseBlockOrNestedIf.as_str(),
            actual: otherwise.rule,
        });
    }

    Ok(())
}

fn without_flat_parallel_prefix(children: &[FlatChild]) -> &[FlatChild] {
    if children
        .first()
        .is_some_and(|child| child.rule == Rule::parallel_kw)
    {
        children.get(1..).unwrap_or_default()
    } else {
        children
    }
}
