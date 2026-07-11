use std::collections::HashMap;
use std::sync::Arc;

use super::{
    MAX_SUMMARY_DEPTH, ReturnSummary, ScalarSource, ScalarValue, SummaryFunctions, ValueSummary,
};
use crate::ReferenceProjection;
use x_ast::{Expression, ExpressionKind, Pattern, Statement, Type};

pub(super) fn summarize_local(
    statement: &Statement,
    bindings: &mut HashMap<String, ValueSummary>,
    depth: &mut usize,
    known: &SummaryFunctions,
) -> Option<()> {
    let Statement::VariableDeclaration {
        pattern: Pattern::Identifier(name),
        value,
        ..
    } = statement
    else {
        return None;
    };
    // Initialisers add summary depth even in flat statement blocks.
    *depth += 1;
    let value = summarize_argument(value, bindings, *depth, known)?;
    bindings.insert(name.clone(), value);

    Some(())
}

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
        ExpressionKind::Block {
            statements,
            result: Some(result),
        } => {
            let mut bindings = bindings.clone();
            let mut depth = depth;

            for statement in statements
                .iter()
                .filter(|statement| !matches!(statement, Statement::Comment(_)))
            {
                summarize_local(statement, &mut bindings, &mut depth, known)?;
            }

            summarize_expression(result, &bindings, depth + 1, known)
        }
        ExpressionKind::FunctionCall {
            target: name,
            arguments: current_arguments,
        } if x_ast::intrinsics::is_catch_call(&name.symbol_name())
            && !bindings.contains_key(name.symbol_name().as_ref())
            && !known.declared_names.contains(name.symbol_name().as_ref()) =>
        {
            let [call, fallback] = current_arguments.as_slice() else {
                return None;
            };

            Some(ReturnSummary::Union(
                Box::new(summarize_expression(call, bindings, depth + 1, known)?),
                Box::new(summarize_expression(fallback, bindings, depth + 1, known)?),
            ))
        }
        ExpressionKind::MethodCall {
            object,
            target: method,
            arguments: current_arguments,
        } if method.symbol_name() == "offset"
            && current_arguments.len() == 1
            && matches!(object.value_type, Type::RawPointer { .. }) =>
        {
            summarize_expression(object, bindings, depth + 1, known)
        }
        ExpressionKind::BinaryOperator { left, operator, .. }
            if matches!(operator, x_ast::Operator::Add | x_ast::Operator::Subtract)
                && matches!(left.value_type, Type::RawPointer { .. }) =>
        {
            summarize_expression(left, bindings, depth + 1, known)
        }
        ExpressionKind::Try { expression } => {
            summarize_expression(expression, bindings, depth + 1, known)
        }
        ExpressionKind::Cast {
            expression,
            to: Type::RawPointer { .. },
        } => summarize_expression(expression, bindings, depth + 1, known),
        ExpressionKind::FunctionCall {
            target: name,
            arguments: current_arguments,
        } if !bindings.contains_key(name.symbol_name().as_ref()) => Some(ReturnSummary::Call {
            callee: known.summaries.get(name.symbol_name().as_ref())?.clone(),
            arguments: current_arguments
                .iter()
                .map(|argument| summarize_argument(argument, bindings, depth + 1, known))
                .collect::<Option<_>>()?,
        }),
        ExpressionKind::Identifier(name) => bindings
            .get(name)
            .map(|value| ReturnSummary::Bound(value.references.clone())),
        ExpressionKind::FieldAccess { object, field } => Some(ReturnSummary::Project(
            Box::new(summarize_expression(object, bindings, depth + 1, known)?),
            ReferenceProjection::from_field_type(&object.value_type, field),
        )),
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
