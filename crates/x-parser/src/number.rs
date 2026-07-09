use std::borrow::Cow;

use pest::iterators::Pair;

use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::parser_messages::{self, ParserIssue};

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
