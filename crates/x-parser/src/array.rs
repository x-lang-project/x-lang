use pest::iterators::Pair;

use crate::source_span_from_pair;
use x_ast::{Expression, ExpressionKind, SourceSpan, StringLiteral, Type};

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
