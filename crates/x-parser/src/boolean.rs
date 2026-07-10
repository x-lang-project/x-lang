use pest::iterators::Pair;

use crate::{Rule, source_span_from_pair};
use x_ast::{Expression, ExpressionKind, Type};

pub fn parse_boolean(pair: Pair<Rule>) -> Expression {
    let boolean_value = pair.as_str() == "true";

    Expression {
        span: Some(source_span_from_pair(&pair)),
        value_type: Type::Boolean,
        kind: ExpressionKind::Boolean(boolean_value),
    }
}
