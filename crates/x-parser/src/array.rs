use pest::iterators::Pair;

use crate::{
    Rule, expression::parse_expression, identifier::parse_identifier,
    lowering_error::LoweringResult, number::parse_number, source_span_from_pair,
};
use x_ast::{Expression, ExpressionKind, SourceSpan, StringLiteral, Type};

pub fn parse_array_literal(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(&pair);
    let mut elements = Vec::new();
    let mut repeat_count = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::expression => {
                if inner_pair.clone().into_inner().next().is_none() {
                    elements.push(Expression {
                        span: Some(source_span_from_pair(&inner_pair)),
                        value_type: Type::String,
                        kind: ExpressionKind::String(StringLiteral { parts: Vec::new() }),
                    });
                } else {
                    elements.push(parse_expression(inner_pair)?);
                }
            }

            Rule::integer => {
                repeat_count = Some(parse_number(inner_pair)?);
            }
            Rule::identifier => {
                repeat_count = Some(parse_identifier(inner_pair));
            }
            _ => {}
        }
    }

    let kind = array_kind(elements, repeat_count, pair_span);

    Ok(Expression {
        span: Some(pair_span),
        value_type: Type::Unknown,
        kind,
    })
}

// Nested semicolons belong to element expressions, not repetition syntax.
fn array_kind(
    mut elements: Vec<Expression>,
    repeat_count: Option<Expression>,
    span: SourceSpan,
) -> ExpressionKind {
    let Some(count) = repeat_count else {
        return ExpressionKind::Array(elements);
    };

    let value = match elements.len() {
        0 => return ExpressionKind::Array(vec![count]),
        1 => elements.remove(0),
        _ => Expression {
            span: Some(span),
            value_type: Type::Unknown,
            kind: ExpressionKind::Array(elements),
        },
    };

    ExpressionKind::ArrayRepeat {
        value: Box::new(value),
        count: Box::new(count),
    }
}
