use pest::iterators::Pair;

use crate::{
    Rule,
    lowering_error::{LoweringError, LoweringResult},
    source_span_from_pair,
};
use x_ast::{Expression, ExpressionKind, StringLiteral, StringPart, Type};
use x_diagnostics::parser_messages::ParserIssue;

pub fn parse_string(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(&pair);
    let text = pair.as_str();
    let content = text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .ok_or_else(|| LoweringError::invalid(&pair, ParserIssue::MissingStringQuotes))?;

    let mut parts = Vec::new();
    let mut current_text = String::new();
    let mut chars = content.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '{' {
            if !current_text.is_empty() {
                parts.push(StringPart::Text(current_text));
                current_text = String::new();
            }

            let mut variable_name = String::new();

            for character in chars.by_ref() {
                if character == '}' {
                    break;
                }

                variable_name.push(character);
            }

            parts.push(StringPart::Interpolation(Box::new(Expression {
                span: None,
                value_type: Type::Unknown,
                kind: ExpressionKind::Identifier(variable_name),
            })));
        } else if character == '\\' {
            match chars.next() {
                Some('n') => current_text.push('\n'),
                Some('r') => current_text.push('\r'),
                Some('t') => current_text.push('\t'),
                Some('\\') => current_text.push('\\'),
                Some('"') => current_text.push('"'),
                Some('0') => current_text.push('\0'),
                Some(other) => {
                    current_text.push('\\');
                    current_text.push(other);
                }
                None => current_text.push('\\'),
            }
        } else {
            current_text.push(character);
        }
    }

    if !current_text.is_empty() {
        parts.push(StringPart::Text(current_text));
    }

    Ok(Expression {
        span: Some(pair_span),
        value_type: Type::String,
        kind: ExpressionKind::String(StringLiteral { parts }),
    })
}
