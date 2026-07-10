use pest::iterators::Pair;

use crate::{Rule, source_span_from_pair};
use x_ast::{Expression, ExpressionKind, Type};

pub fn parse_identifier(pair: Pair<Rule>) -> Expression {
    Expression {
        span: Some(source_span_from_pair(&pair)),
        value_type: Type::Unknown,
        kind: ExpressionKind::Identifier(pair.as_str().trim().to_string()),
    }
}
