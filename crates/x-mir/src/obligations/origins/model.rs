use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::LocalIdentifier;
use x_ast::Type;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::obligations) enum Step {
    Field(String),
    Index(Option<i64>),
    Dereference,
    Unsupported,
}
