use crate::{Expression, ExpressionKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantBinding {
    pub name: String,
}

impl InvariantBinding {
    pub fn source_receiver() -> Self {
        Self {
            name: "self".into(),
        }
    }

    pub fn refers_to(&self, expression: &Expression) -> bool {
        matches!(&expression.kind, ExpressionKind::Identifier(name) if name == &self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructInvariant {
    pub receiver: InvariantBinding,
    pub predicate: Box<Expression>,
}

impl StructInvariant {
    pub fn source(predicate: Expression) -> Self {
        Self {
            receiver: InvariantBinding::source_receiver(),
            predicate: Box::new(predicate),
        }
    }
}
