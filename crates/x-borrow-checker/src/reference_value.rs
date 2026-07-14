use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReferenceProjection {
    Field(String),
    Index(i64),
    AnyIndex,
}

impl ReferenceProjection {
    pub(crate) fn from_field_type(value_type: &x_ast::Type, name: &str) -> Self {
        let mut value_type = value_type;

        while let x_ast::Type::Reference { inner, .. } = value_type {
            value_type = inner;
        }

        if matches!(value_type, x_ast::Type::Tuple(_))
            && let Ok(index) = name.parse()
        {
            return Self::Index(index);
        }

        Self::Field(name.into())
    }

    pub(crate) fn from_index_expression(expression: &x_ast::Expression) -> Self {
        match expression.kind {
            x_ast::ExpressionKind::Integer(index) => Self::Index(index),
            _ => Self::AnyIndex,
        }
    }
}

impl std::fmt::Display for ReferenceProjection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(name) => formatter.write_str(name),
            Self::Index(index) => index.fmt(formatter),
            Self::AnyIndex => formatter.write_str("*"),
        }
    }
}
