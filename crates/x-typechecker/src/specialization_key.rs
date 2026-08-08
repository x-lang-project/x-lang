use x_ast::{Expression, ExpressionKind, StringPart, UnaryOperator};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum StaticValue {
    Integer(i64),
    Float(u64),
    Boolean(bool),
    Character(char),
    String(String),
    Negated(Box<StaticValue>),
}

impl StaticValue {
    pub(super) fn from_expression(expression: &Expression) -> Option<Self> {
        Some(match &expression.kind {
            _ => return None,
        })
    }
}
