use super::{CstValidationError, FlatChild, FlatNodeContext, Rule};
use x_diagnostics::parser_messages::CstMessage;

pub(super) fn flat_require_rules(
    context: FlatNodeContext<'_>,
    expected_rules: &[Rule],
    expected_description: &'static str,
) -> Result<(), CstValidationError> {
    let FlatNodeContext { parent, .. } = context;

    for (index, expected) in expected_rules.iter().copied().enumerate() {
        let child = flat_require_child(context, index, expected_description)?;

        if child.rule != expected {
            return Err(CstValidationError::UnexpectedChild {
                offset: child.offset,
                parent,
                expected: expected_description,
                actual: child.rule,
            });
        }
    }

    Ok(())
}

pub(super) fn flat_require_any_rule(
    context: FlatNodeContext<'_>,
    expected_rule: Rule,
    expected: &'static str,
) -> Result<(), CstValidationError> {
    flat_require_any_of(context, &[expected_rule], expected)
}

pub(super) fn flat_require_fields(
    context: FlatNodeContext<'_>,
    fields: &[(&[Rule], CstMessage)],
) -> Result<(), CstValidationError> {
    for (rules, message) in fields {
        flat_require_any_of(context, rules, message.as_str())?;
    }

    Ok(())
}

pub(super) fn flat_require_suffixes(
    context: FlatNodeContext<'_>,
    first: (Rule, CstMessage),
    suffix: (Rule, CstMessage),
) -> Result<(), CstValidationError> {
    flat_require_first_rule(context, first.0, first.1.as_str())?;

    flat_require_remaining_rules(context, 1, &[suffix.0], suffix.1.as_str())
}

pub(super) fn flat_require_any_of(
    context: FlatNodeContext<'_>,
    expected_rules: &[Rule],
    expected: &'static str,
) -> Result<(), CstValidationError> {
    let FlatNodeContext {
        parent,
        offset,
        children,
    } = context;

    if children
        .iter()
        .any(|child| expected_rules.contains(&child.rule))
    {
        Ok(())
    } else {
        Err(CstValidationError::MissingChild {
            offset,
            parent,
            expected,
        })
    }
}

pub(super) fn flat_require_first_rule(
    context: FlatNodeContext<'_>,
    expected_rule: Rule,
    expected: &'static str,
) -> Result<(), CstValidationError> {
    flat_require_first_of(context, &[expected_rule], expected)
}

pub(super) fn flat_require_first_of(
    context: FlatNodeContext<'_>,
    expected_rules: &[Rule],
    expected: &'static str,
) -> Result<(), CstValidationError> {
    let FlatNodeContext { parent, .. } = context;
    let child = flat_require_child(context, 0, expected)?;

    if expected_rules.contains(&child.rule) {
        Ok(())
    } else {
        Err(CstValidationError::UnexpectedChild {
            offset: child.offset,
            parent,
            expected,
            actual: child.rule,
        })
    }
}

pub(super) fn flat_require_remaining_rules(
    context: FlatNodeContext<'_>,
    start: usize,
    allowed: &[Rule],
    expected: &'static str,
) -> Result<(), CstValidationError> {
    let FlatNodeContext {
        parent,
        offset,
        children,
    } = context;
    let remaining = children
        .get(start..)
        .ok_or(CstValidationError::MissingChild {
            offset,
            parent,
            expected,
        })?;

    if let Some(child) = remaining
        .iter()
        .find(|child| !allowed.contains(&child.rule))
    {
        Err(CstValidationError::UnexpectedChild {
            offset: child.offset,
            parent,
            expected,
            actual: child.rule,
        })
    } else {
        Ok(())
    }
}

pub(super) fn flat_require_all_rules(
    context: FlatNodeContext<'_>,
    expected_rule: Rule,
    expected: &'static str,
) -> Result<(), CstValidationError> {
    flat_require_child(context, 0, expected)?;

    flat_require_remaining_rules(context, 0, &[expected_rule], expected)
}

pub(super) fn flat_require_single_of(
    context: FlatNodeContext<'_>,
    expected_rules: &[Rule],
    expected: &'static str,
) -> Result<(), CstValidationError> {
    let FlatNodeContext {
        parent, children, ..
    } = context;
    let child = flat_require_child(context, 0, expected)?;

    if children.len() == 1 && expected_rules.contains(&child.rule) {
        Ok(())
    } else {
        Err(CstValidationError::UnexpectedChild {
            offset: child.offset,
            parent,
            expected,
            actual: child.rule,
        })
    }
}

pub(super) fn flat_require_child<'a>(
    context: FlatNodeContext<'a>,
    index: usize,
    expected: &'static str,
) -> Result<&'a FlatChild, CstValidationError> {
    let FlatNodeContext {
        parent,
        offset,
        children,
    } = context;

    children.get(index).ok_or(CstValidationError::MissingChild {
        offset,
        parent,
        expected,
    })
}

pub(super) fn validate_flat_binary_chain(
    context: FlatNodeContext<'_>,
    operand: Rule,
    operators: &[Rule],
) -> Result<(), CstValidationError> {
    let FlatNodeContext {
        parent,
        offset,
        children,
    } = context;

    if children.is_empty() || children.len().is_multiple_of(2) {
        return Err(CstValidationError::MissingChild {
            offset,
            parent,
            expected: CstMessage::OddLengthOperandOperatorChain.as_str(),
        });
    }

    for (index, child) in children.iter().enumerate() {
        let valid = if index % 2 == 0 {
            child.rule == operand
        } else {
            operators.contains(&child.rule)
        };

        if !valid {
            return Err(CstValidationError::UnexpectedChild {
                offset: child.offset,
                parent,
                expected: CstMessage::AlternatingOperandsAndOperators.as_str(),
                actual: child.rule,
            });
        }
    }

    Ok(())
}
