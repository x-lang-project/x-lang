use std::slice;

use super::{
    ArgumentValue, MAX_SUMMARY_DEPTH, MAX_SUMMARY_WORK, ReturnSummary, ScalarSource, ScalarValue,
};
use crate::ReferenceProjection;
use crate::ReferenceValue;
use x_ast::Expression;

impl ReturnSummary {
    pub(crate) fn instantiate(
        &self,
        arguments: &[ReferenceValue],
        expressions: &[Expression],
    ) -> ReferenceValue {
        let arguments = arguments
            .iter()
            .enumerate()
            .map(|(index, references)| ArgumentValue {
                references: references.clone(),
                scalar: expressions
                    .get(index)
                    .and_then(ScalarValue::from_expression),
            })
            .collect::<Vec<_>>();

        let mut budget = MAX_SUMMARY_WORK;

        self.instantiate_inner(&arguments, &mut budget, 0)
    }

    pub(super) fn instantiate_inner(
        &self,
        arguments: &[ArgumentValue],
        budget: &mut usize,
        depth: usize,
    ) -> ReferenceValue {
        if *budget == 0 || depth >= MAX_SUMMARY_DEPTH {
            return ReferenceValue::default();
        }

        *budget -= 1;

        match self {
            Self::Parameter(index) => arguments
                .get(*index)
                .map(|argument| argument.references.clone())
                .unwrap_or_default(),
            Self::Bound(value) => value.instantiate_inner(arguments, budget, depth + 1),
            Self::Project(base, field) => base
                .instantiate_inner(arguments, budget, depth + 1)
                .project(slice::from_ref(field)),
            Self::Aggregate(fields) => ReferenceValue {
                fields: fields
                    .iter()
                    .map(|(name, value)| {
                        (
                            name.clone(),
                            value.instantiate_inner(arguments, budget, depth + 1),
                        )
                    })
                    .collect(),
                ..Default::default()
            },
            Self::Select(index, left, right) => {
                match arguments.get(*index).and_then(|argument| argument.scalar) {
                    Some(ScalarValue::Boolean(true)) => {
                        left.instantiate_inner(arguments, budget, depth + 1)
                    }
                    Some(ScalarValue::Boolean(false)) => {
                        right.instantiate_inner(arguments, budget, depth + 1)
                    }
                    _ => left
                        .instantiate_inner(arguments, budget, depth + 1)
                        .join(&right.instantiate_inner(arguments, budget, depth + 1)),
                }
            }
            Self::Index(base, index) => {
                let field = match arguments.get(*index).and_then(|argument| argument.scalar) {
                    Some(ScalarValue::Integer(index)) => ReferenceProjection::Index(index),
                    _ => ReferenceProjection::AnyIndex,
                };

                base.instantiate_inner(arguments, budget, depth + 1)
                    .project(&[field])
            }
            Self::Call {
                callee,
                arguments: inputs,
            } => {
                let values = inputs
                    .iter()
                    .map(|input| ArgumentValue {
                        references: input.references.instantiate_inner(
                            arguments,
                            budget,
                            depth + 1,
                        ),
                        scalar: match input.scalar {
                            ScalarSource::Parameter(index) => {
                                arguments.get(index).and_then(|argument| argument.scalar)
                            }
                            ScalarSource::Literal(value) => Some(value),
                            ScalarSource::Unknown => None,
                        },
                    })
                    .collect::<Vec<_>>();

                callee.instantiate_inner(&values, budget, depth + 1)
            }
            Self::Empty => ReferenceValue::default(),
            Self::Never => ReferenceValue {
                origin_known: true,
                is_unreachable: true,
                ..Default::default()
            },
            Self::Union(left, right) => left
                .instantiate_inner(arguments, budget, depth + 1)
                .join(&right.instantiate_inner(arguments, budget, depth + 1)),
        }
    }
}
