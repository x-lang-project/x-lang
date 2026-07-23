use std::collections::BTreeSet;

use super::{
    Analysis, Records, access,
    model::{Origin, State, Step, Value, unknown},
};
use crate::{CallSite, MirBody, Operand, ResolvedCall, Rvalue};
use x_ast::Type;

pub(super) fn operand(
    body: &MirBody,
    operand: &Operand,
    state: &State,
    records: &Records<'_>,
) -> Value {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => access::read(body, place, state, records),
        _ => Value::default(),
    }
}

pub(super) fn call_value(analysis: &Analysis<'_, '_>, call: CallSite<'_>, state: &State) -> Value {
    let body = analysis.body;
    let records = analysis.records;
    let summaries = analysis.summaries;
    let function = call.name;
    let arguments = call.arguments;
    let Some(summary) = summaries.get(function) else {
        let value_type = match body.call_targets.get(function) {
            Some(ResolvedCall::Source {
                return_type: Some(value_type),
                ..
            }) => value_type,
            _ => &Type::Void,
        };

        return unknown(value_type, records);
    };
    // A mutating call can invalidate a pre-call pointee projection.
    if super::havoc::may_mutate(analysis, call) {
        let mut result = summary.clone();

        for origins in result.0.values_mut() {
            *origins = BTreeSet::from([Origin::Unknown]);
        }

        return result;
    }

    let mut result = Value::default();

    for (path, origins) in &summary.0 {
        let mut substituted = BTreeSet::new();

        for origin in origins {
            match origin {
                Origin::Parameter { index, path } => {
                    if let Some(argument) = arguments.get(*index) {
                        substituted
                            .extend(operand(body, argument, state, records).select(path).root());
                    } else {
                        substituted.insert(Origin::Unknown);
                    }
                }
                Origin::Local { .. } => {
                    substituted.insert(Origin::Unknown);
                }
                other => {
                    substituted.insert(other.clone());
                }
            }
        }

        result.0.insert(path.clone(), substituted);
    }

    result
}

pub(super) fn rvalue(analysis: &Analysis<'_, '_>, value: &Rvalue, state: &State) -> Value {
    let body = analysis.body;
    let records = analysis.records;

    match value {
        Rvalue::StructInstantiate { fields, .. } => {
            let mut result = Value::default();

            for (name, value) in fields {
                result.join(
                    &operand(body, value, state, records).prefixed(&[Step::Field(name.clone())]),
                );
            }

            result
        }
        Rvalue::Tuple(values) => {
            let mut result = Value::default();

            for (index, value) in values.iter().enumerate() {
                result.join(
                    &operand(body, value, state, records)
                        .prefixed(&[Step::Field(index.to_string())]),
                );
            }

            result
        }
        Rvalue::Array(values) => {
            let mut result = Value::default();

            for (index, value) in values.iter().enumerate() {
                result.join(
                    &operand(body, value, state, records)
                        .prefixed(&[Step::Index(Some(index as i64))]),
                );
            }

            result
        }
        Rvalue::Call {
            function,
            arguments: current_arguments,
        }
        | Rvalue::TryCall {
            function,
            arguments: current_arguments,
        } => call_value(
            analysis,
            crate::CallSite {
                name: function,
                arguments: current_arguments,
                destination: None,
            },
            state,
        ),
        Rvalue::CatchCall {
            function,
            arguments: current_arguments,
            fallback,
        } => {
            let mut result = call_value(
                analysis,
                crate::CallSite {
                    name: function,
                    arguments: current_arguments,
                    destination: None,
                },
                state,
            );
            result.join(&operand(body, fallback, state, records));

            result
        }
        _ => Value::default(),
    }
}
