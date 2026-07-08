use super::Attempt;
use crate::integer_suffix::IntegerSuffix;
use x_ast::{ExpressionKind, Type};

pub(super) fn numeric(raw: &str) -> Attempt<(Type, ExpressionKind)> {
    let normalized = raw.replace('_', "");
    let (digits, suffix) = IntegerSuffix::split(&normalized);

    if matches!(suffix, Some(IntegerSuffix::I128 | IntegerSuffix::U128)) {
        return None;
    }

    let resolved_type = suffix.map_or(Type::Unknown, IntegerSuffix::value_type);
    let integer = if let Some(hex) = digits.strip_prefix("0x") {
        i64::from_str_radix(hex, 16).ok()
    } else if let Some(bin) = digits.strip_prefix("0b") {
        i64::from_str_radix(bin, 2).ok()
    } else {
        digits.parse::<i64>().ok()
    };

    if let Some(value) = integer {
        return Some((resolved_type, ExpressionKind::Integer(value)));
    }

    if normalized.starts_with("0x")
        || normalized.starts_with("0b")
        || !normalized.contains(['.', 'e', 'E'])
    {
        return None;
    }

    let value = normalized.parse::<f64>().ok()?;

    value
        .is_finite()
        .then_some((Type::F64, ExpressionKind::Float(value)))
}

pub(super) fn decode_string(content: &str) -> Attempt<String> {
    let mut result = String::new();
    let mut chars = content.chars();

    while let Some(character) = chars.next() {
        if character == '\\' {
            match chars.next()? {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                '0' => result.push('\0'),
                other => {
                    result.push('\\');
                    result.push(other);
                }
            }
        } else {
            result.push(character);
        }
    }

    Some(result)
}
