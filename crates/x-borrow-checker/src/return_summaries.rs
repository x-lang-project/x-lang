use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::ReferenceProjection;
use x_ast::{Expression, ExpressionKind};

mod construction;

mod expressions;

// Shared summary nodes still consume work on each visit.
const MAX_SUMMARY_DEPTH: usize = 128;

#[derive(Clone, Debug)]
pub(super) enum ReturnSummary {
    Parameter(usize),
    Bound(Arc<Self>),
    Project(Box<Self>, ReferenceProjection),
    Aggregate(Vec<(ReferenceProjection, Self)>),
    Select(usize, Box<Self>, Box<Self>),
    Index(Box<Self>, usize),
    Call {
        callee: Arc<Self>,
        arguments: Vec<ValueSummary>,
    },
    Empty,
    Never,
    Union(Box<Self>, Box<Self>),
}

#[derive(Clone, Copy, Debug)]
enum ScalarValue {
    Boolean(bool),
    Integer(i64),
}

impl ScalarValue {
    fn from_expression(expression: &Expression) -> Option<Self> {
        match expression.kind {
            ExpressionKind::Boolean(value) => Some(Self::Boolean(value)),
            ExpressionKind::Integer(value) => Some(Self::Integer(value)),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
enum ScalarSource {
    Parameter(usize),
    Literal(ScalarValue),
    Unknown,
}

#[derive(Clone, Debug)]
pub(super) struct ValueSummary {
    references: Arc<ReturnSummary>,
    scalar: ScalarSource,
}

struct SummaryFunctions {
    summaries: HashMap<String, Arc<ReturnSummary>>,
    declared_names: HashSet<String>,
}
