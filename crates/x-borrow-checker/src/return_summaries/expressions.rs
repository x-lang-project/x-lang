use std::collections::HashMap;
use std::sync::Arc;

use super::{
    MAX_SUMMARY_DEPTH, ReturnSummary, ScalarSource, ScalarValue, SummaryFunctions, ValueSummary,
};
use crate::ReferenceProjection;
use x_ast::{Expression, ExpressionKind, Pattern, Statement, Type};

pub(super) fn summarize_expression(
    expression: &Expression,
    bindings: &HashMap<String, ValueSummary>,
    depth: usize,
    known: &SummaryFunctions,
) -> Option<ReturnSummary> {
    if depth >= MAX_SUMMARY_DEPTH {
        return None;
    }

    match &expression.kind {
        ExpressionKind::ArrayAccess { array, index } => {
            let base = Box::new(summarize_expression(array, bindings, depth + 1, known)?);

            match scalar_source(index, bindings) {
                ScalarSource::Literal(ScalarValue::Integer(index)) => Some(ReturnSummary::Project(
                    base,
                    ReferenceProjection::Index(index),
                )),
                ScalarSource::Parameter(index) => Some(ReturnSummary::Index(base, index)),
                _ => None,
            }
        }
        ExpressionKind::StructInstantiate(initialise) => Some(ReturnSummary::Aggregate(
            initialise
                .fields
                .iter()
                .map(|(field, value)| {
                    Some((
                        ReferenceProjection::Field(field.clone()),
                        summarize_expression(value, bindings, depth + 1, known)?,
                    ))
                })
                .collect::<Option<_>>()?,
        )),
        ExpressionKind::Tuple(items) | ExpressionKind::Array(items) => {
            Some(ReturnSummary::Aggregate(
                items
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        Some((
                            ReferenceProjection::Index(index as i64),
                            summarize_expression(value, bindings, depth + 1, known)?,
                        ))
                    })
                    .collect::<Option<_>>()?,
            ))
        }
        ExpressionKind::Integer(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Boolean(_)
        | ExpressionKind::String(_)
        | ExpressionKind::Character(_) => Some(ReturnSummary::Empty),
        _ => None,
    }
}

pub(super) fn scalar_source(
    expression: &Expression,
    bindings: &HashMap<String, ValueSummary>,
) -> ScalarSource {
    match &expression.kind {
        ExpressionKind::Identifier(name) => bindings
            .get(name)
            .map(|value| value.scalar.clone())
            .unwrap_or(ScalarSource::Unknown),
        _ => ScalarValue::from_expression(expression)
            .map(ScalarSource::Literal)
            .unwrap_or(ScalarSource::Unknown),
    }
}

pub(super) fn summarize_argument(
    expression: &Expression,
    bindings: &HashMap<String, ValueSummary>,
    depth: usize,
    known: &SummaryFunctions,
) -> Option<ValueSummary> {
    if depth >= MAX_SUMMARY_DEPTH {
        return None;
    }

    if let ExpressionKind::Identifier(name) = &expression.kind {
        return bindings.get(name).cloned();
    }

    Some(ValueSummary {
        references: Arc::new(summarize_expression(expression, bindings, depth, known)?),
        scalar: scalar_source(expression, bindings),
    })
}
