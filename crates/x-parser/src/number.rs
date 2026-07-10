use std::borrow::Cow;

use pest::iterators::Pair;

use crate::{
    Rule,
    lowering_error::{LoweringError, LoweringResult},
    source_span_from_pair,
};
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::parser_messages::{self, ParserIssue};

pub fn parse_number(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(&pair);
    let literal = pair.as_str();
    let normalized = if literal.contains('_') {
        Cow::Owned(literal.replace('_', ""))
    } else {
        Cow::Borrowed(literal)
    };

    let number_str = normalized.as_ref();
    let numeric_rule = pair
        .clone()
        .into_inner()
        .next()
        .map_or(pair.as_rule(), |child| child.as_rule());
    let (digits, value_type) = split_integer_suffix(number_str);

    if numeric_rule != Rule::float {
        if value_type == Type::Unknown
            && crate::integer_suffix::IntegerSuffix::split(number_str)
                .1
                .is_some()
        {
            return Err(LoweringError::invalid(
                &pair,
                parser_messages::unsupported_integer_suffix(),
            ));
        }

        let index = parse_radix_integer(digits)
            .or_else(|| digits.parse::<i64>().ok())
            .ok_or_else(|| {
                LoweringError::invalid(&pair, parser_messages::integer_representation_overflow())
            })?;

        Ok(Expression {
            span: Some(pair_span),
            value_type,
            kind: ExpressionKind::Integer(index),
        })
    } else {
        let function = number_str.parse::<f64>().map_err(|error| {
            LoweringError::invalid(&pair, ParserIssue::InvalidNumericLiteral(error.to_string()))
        })?;

        if !function.is_finite() {
            return Err(LoweringError::invalid(
                &pair,
                ParserIssue::NonFiniteNumericLiteral,
            ));
        }

        Ok(Expression {
            span: Some(pair_span),
            value_type: Type::F64,
            kind: ExpressionKind::Float(function),
        })
    }
}

fn split_integer_suffix(number_str: &str) -> (&str, Type) {
    let (digits, suffix) = crate::integer_suffix::IntegerSuffix::split(number_str);

    (
        digits,
        suffix.map_or(Type::Unknown, |suffix| suffix.value_type()),
    )
}

fn parse_radix_integer(number_str: &str) -> Option<i64> {
    let (sign, unsigned) = number_str
        .strip_prefix('-')
        .map_or((1, number_str), |rest| (-1, rest));
    let (radix, digits) = if let Some(rest) = unsigned.strip_prefix("0x") {
        (16, rest)
    } else {
        (2, unsigned.strip_prefix("0b")?)
    };

    i128::from_str_radix(digits, radix)
        .ok()
        .and_then(|value| i64::try_from(sign * value).ok())
}
