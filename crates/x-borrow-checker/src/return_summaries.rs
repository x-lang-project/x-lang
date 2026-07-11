use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::ReferenceProjection;
use x_ast::{Expression, ExpressionKind};

// Shared summary nodes still consume work on each visit.

#[derive(Clone, Copy, Debug)]
enum ScalarValue {
    Boolean(bool),
    Integer(i64),
}

#[derive(Clone, Debug)]
enum ScalarSource {
    Parameter(usize),
    Literal(ScalarValue),
    Unknown,
}
