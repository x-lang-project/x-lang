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
}
