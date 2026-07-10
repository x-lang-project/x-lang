mod contracts;
mod control;
mod declarations;
mod expressions;
use std::fmt;

use pest::iterators::Pair;

use crate::Rule;
use contracts::{flat_require_child, validate_flat_binary_chain};
use declarations::validate_external_block_function;
use expressions::{binary_chain_contract, flat_validate_array_repetition};
use x_diagnostics::ParserDiagnostic;
use x_diagnostics::parser_messages::{self, CstMessage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CstValidationError {
    MissingChild {
        offset: usize,
        parent: Rule,
        expected: &'static str,
    },
    UnexpectedChild {
        offset: usize,
        parent: Rule,
        expected: &'static str,
        actual: Rule,
    },
    UnsupportedShape {
        offset: usize,
        rule: Rule,
        detail: &'static str,
    },
}

impl fmt::Display for CstValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingChild {
                offset,
                parent,
                expected,
            } => parser_messages::cst_missing_message(*offset, parent, expected),
            Self::UnexpectedChild {
                offset,
                parent,
                expected,
                actual,
            } => parser_messages::cst_unexpected_message(*offset, parent, expected, actual),
            Self::UnsupportedShape {
                offset,
                rule,
                detail,
            } => parser_messages::cst_unsupported_message(*offset, rule, detail),
        };

        formatter.write_str(&message)
    }
}

#[derive(Debug)]
pub(crate) struct ValidatedProgramCst<'input>(Pair<'input, Rule>);

impl<'input> ValidatedProgramCst<'input> {
    pub(crate) fn into_pair(self) -> Pair<'input, Rule> {
        self.0
    }
}

pub(crate) fn validate_program_cst<'input>(
    program: &Pair<'input, Rule>,
) -> Result<ValidatedProgramCst<'input>, CstValidationError> {
    let mut pending = vec![program.clone()];
    let mut children = Vec::new();

    while let Some(pair) = pending.pop() {
        children.clear();
        for child in pair.clone().into_inner() {
            children.push(FlatChild {
                rule: child.as_rule(),
                offset: child.as_span().start(),
            });
            pending.push(child);
        }

        validate_pair_shape(&pair, &children)?;

        if !matches!(
            pair.as_rule(),
            Rule::function_definition | Rule::struct_declaration
        ) && let Some(attribute) = children.iter().find(|child| child.rule == Rule::attribute)
        {
            return Err(CstValidationError::UnsupportedShape {
                offset: attribute.offset,
                rule: Rule::attribute,
                detail: CstMessage::UnsupportedAttributeTarget.as_str(),
            });
        }
    }

    Ok(ValidatedProgramCst(program.clone()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FlatChild {
    rule: Rule,
    offset: usize,
}

#[derive(Clone, Copy)]
struct FlatNodeContext<'a> {
    parent: Rule,
    offset: usize,
    children: &'a [FlatChild],
}

impl<'a> FlatNodeContext<'a> {
    fn new(parent: Rule, offset: usize, children: &'a [FlatChild]) -> Self {
        Self {
            parent,
            offset,
            children,
        }
    }
}

fn validate_flat_shape(
    rule: Rule,
    offset: usize,
    children: &[FlatChild],
) -> Option<Result<(), CstValidationError>> {
    if let Some((operand, operators)) = binary_chain_contract(rule) {
        return Some(validate_flat_binary_chain(
            FlatNodeContext::new(rule, offset, children),
            operand,
            operators,
        ));
    }

    control::validate(rule, offset, children)
        .or_else(|| expressions::validate(rule, offset, children))
        .or_else(|| declarations::validate(rule, offset, children))
}

fn validate_text_sensitive_flat_shape(
    rule: Rule,
    text: &str,
    offset: usize,
    children: &[FlatChild],
) -> Option<Result<(), CstValidationError>> {
    let result = match rule {
        Rule::pattern if text.trim() != "_" => flat_require_child(
            FlatNodeContext::new(rule, offset, children),
            0,
            CstMessage::InnerPattern.as_str(),
        )
        .map(|_| ()),
        Rule::pattern => Ok(()),
        Rule::array_literal if text.contains(';') => {
            flat_validate_array_repetition(offset, children)
        }
        Rule::array_literal => Ok(()),
        Rule::import_path if text.split("::").all(str::is_empty) => {
            Err(CstValidationError::MissingChild {
                offset,
                parent: rule,
                expected: CstMessage::NonEmptyImportPath.as_str(),
            })
        }
        Rule::import_path => Ok(()),
        _ => return None,
    };

    Some(result)
}

fn validate_pair_shape(
    pair: &Pair<Rule>,
    flat_children: &[FlatChild],
) -> Result<(), CstValidationError> {
    let rule = pair.as_rule();
    let offset = pair.as_span().start();

    if let Some(result) = validate_flat_shape(rule, offset, flat_children) {
        result?;
    }

    if let Some(result) =
        validate_text_sensitive_flat_shape(rule, pair.as_str(), offset, flat_children)
    {
        result?;
    }

    match rule {
        Rule::basic_type if matches!(pair.as_str(), "i128" | "u128") => {
            return Err(CstValidationError::UnsupportedShape {
                offset,
                rule,
                detail: CstMessage::UnsupportedWideInteger.as_str(),
            });
        }
        Rule::integer if pair.as_str().ends_with("i128") || pair.as_str().ends_with("u128") => {
            return Err(CstValidationError::UnsupportedShape {
                offset,
                rule,
                detail: CstMessage::UnsupportedWideIntegerLiteral.as_str(),
            });
        }
        Rule::external_block_function => {
            validate_external_block_function(offset, pair.clone().into_inner())?
        }
        _ => {}
    }

    Ok(())
}

impl CstValidationError {
    pub(crate) fn diagnostic(&self) -> ParserDiagnostic {
        match self {
            Self::MissingChild { .. } => ParserDiagnostic::MissingCstChild,
            Self::UnexpectedChild { .. } => ParserDiagnostic::UnexpectedCstChild,
            Self::UnsupportedShape { .. } => ParserDiagnostic::UnsupportedCstShape,
        }
    }
}
