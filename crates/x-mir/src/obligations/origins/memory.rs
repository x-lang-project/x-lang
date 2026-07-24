use std::collections;
use std::slice;

use std::collections::BTreeMap;

use super::{
    Analysis, Records, Summaries, access, flow,
    model::{Origin, State},
};
use crate::obligations::Function;
use crate::{MirBody, MirStatement, MirTerminator, Operand, Place, Rvalue};

fn place(body: &MirBody, value: &Place, state: &State, records: &Records<'_>) -> bool {
    let origins = access::storage(body, value, state, records);
    let observable = origins.is_empty()
        || origins
            .iter()
            .any(|origin| !matches!(origin, Origin::Local { .. }));

    observable
        || match value {
            Place::Index { index, .. } => operand(body, index, state, records),
            _ => false,
        }
}

fn operand(body: &MirBody, value: &Operand, state: &State, records: &Records<'_>) -> bool {
    match value {
        Operand::Copy(value) | Operand::Move(value) => place(body, value, state, records),
        _ => false,
    }
}

fn rvalue(body: &MirBody, value: &Rvalue, state: &State, records: &Records<'_>) -> bool {
    let check = |value| operand(body, value, state, records);

    match value {
        Rvalue::Use(value)
        | Rvalue::UnaryOperator(_, value)
        | Rvalue::Cast {
            expression: value, ..
        } => check(value),
        Rvalue::BinaryOperator(_, first_value, second_value)
        | Rvalue::PointerOffset {
            base: first_value,
            offset: second_value,
        } => check(first_value) || check(second_value),
        Rvalue::Tuple(values) | Rvalue::Array(values) => values.iter().any(check),
        Rvalue::Call { arguments, .. } | Rvalue::TryCall { arguments, .. } => {
            arguments.iter().any(check)
        }
        Rvalue::CatchCall {
            arguments,
            fallback,
            ..
        } => arguments.iter().any(check) || check(fallback),
        Rvalue::StructInstantiate { fields, .. } => fields.iter().any(|(_, value)| check(value)),
        Rvalue::AddressOf { .. } | Rvalue::SizeOf(_) => false,
    }
}

fn terminator(body: &MirBody, value: &MirTerminator, state: &State, records: &Records<'_>) -> bool {
    let check = |value| operand(body, value, state, records);

    match value {
        MirTerminator::SwitchInteger { discriminant, .. } => check(discriminant),
        MirTerminator::Call {
            arguments,
            destination,
            ..
        }
        | MirTerminator::TryCall {
            arguments,
            destination,
            ..
        } => arguments.iter().any(check) || place(body, destination, state, records),
        MirTerminator::CatchCall {
            arguments,
            destination,
            fallback,
            ..
        } => {
            arguments.iter().any(check)
                || check(fallback)
                || place(body, destination, state, records)
        }
        MirTerminator::TailCall { arguments, .. } => arguments.iter().any(check),
        MirTerminator::Throw { error } => check(error),
        MirTerminator::Exit { code } => check(code),
        _ => false,
    }
}

pub(in crate::obligations) fn unproved_memory(
    functions: &[Function<'_>],
    records: &Records<'_>,
    summaries: &Summaries,
    mutating_calls: &collections::BTreeSet<String>,
) -> BTreeMap<String, bool> {
    functions
        .iter()
        .map(|function| {
            let unproved = function.body.is_none_or(|body| {
                let analysis = Analysis {
                    body,
                    records,
                    summaries,
                    mutating_calls,
                };

                flow::states(&analysis)
                    .into_iter()
                    .any(|(identifier, mut state)| {
                        let Some(block) = body.basic_blocks.get(&identifier) else {
                            return true;
                        };

                        for statement in &block.statements {
                            let observable = match statement {
                                MirStatement::Assign(destination, value) => {
                                    place(body, destination, &state, records)
                                        || rvalue(body, value, &state, records)
                                }
                                MirStatement::ParallelFor {
                                    start,
                                    end,
                                    captures,
                                    ..
                                } => {
                                    operand(body, start, &state, records)
                                        || operand(body, end, &state, records)
                                        || captures
                                            .iter()
                                            .any(|value| operand(body, value, &state, records))
                                }
                                _ => false,
                            };

                            if observable {
                                return true;
                            }

                            state = flow::transfer_statements(
                                &analysis,
                                slice::from_ref(statement),
                                state,
                            );
                        }

                        block
                            .terminator
                            .as_ref()
                            .is_some_and(|term| terminator(body, term, &state, records))
                    })
            });

            (function.name.to_owned(), unproved)
        })
        .collect()
}
