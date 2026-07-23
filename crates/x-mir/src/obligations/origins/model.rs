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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::obligations) enum Origin {
    Parameter { index: usize, path: Vec<Step> },
    Local { identifier: usize, path: Vec<Step> },
    Global(String),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(in crate::obligations) struct Value(pub BTreeMap<Vec<Step>, BTreeSet<Origin>>);

pub(super) type State = HashMap<LocalIdentifier, Value>;
