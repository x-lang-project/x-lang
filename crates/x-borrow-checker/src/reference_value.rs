use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReferenceProjection {
    Field(String),
    Index(i64),
    AnyIndex,
}
