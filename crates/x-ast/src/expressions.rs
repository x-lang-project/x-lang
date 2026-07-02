use std::fmt;

use crate::Operator;
use crate::SourceSpan;
use crate::Type;
use crate::UnaryOperator;

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Identifier(String),
    Tuple(Vec<String>),
    Enum {
        variant: String,
        bindings: Vec<String>,
    },
}
