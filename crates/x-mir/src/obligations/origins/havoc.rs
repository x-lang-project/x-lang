use std::collections::{BTreeSet, HashMap};

use super::{
    Analysis, Records,
    model::{Origin, State, type_paths},
};
use crate::obligations::Function;
use crate::{CallSite, MirBody, MirStatement, MirTerminator, Place};
use x_ast::Type;

pub(super) fn may_mutate(analysis: &Analysis<'_, '_>, call: CallSite<'_>) -> bool {
    // Opaque callees can mutate escaped pointers through globals.
    !analysis.summaries.contains_key(call.name) || analysis.mutating_calls.contains(call.name)
}

/// Scalar raw pointers can still alias reference slots.
fn raw_type(value_type: &Type) -> bool {
    match value_type {
        Type::RawPointer { .. } | Type::Unknown | Type::Infer(_) => true,
        Type::Reference { inner, .. } => raw_type(inner),
        _ => false,
    }
}

pub(super) fn raw_place(body: &MirBody, place: &Place) -> bool {
    match place {
        Place::Field { base, .. } | Place::Dereference { base, .. } | Place::Index { base, .. } => {
            raw_type(&body.place_type(base)) || raw_place(body, base)
        }
        _ => false,
    }
}

fn indirect(body: &MirBody, place: &Place) -> bool {
    match place {
        Place::Local(_) => false,
        Place::Global { .. } | Place::Dereference { .. } => true,
        Place::Field { base, .. } | Place::Index { base, .. } => {
            body.place_type(base).pointee().is_some() || indirect(body, base)
        }
    }
}

fn reference_write(body: &MirBody, place: &Place, records: &Records<'_>) -> bool {
    indirect(body, place)
        && (raw_place(body, place) || !type_paths(&body.place_type(place), records).is_empty())
}

pub(super) fn mutating_calls(
    functions: &[Function<'_>],
    records: &Records<'_>,
) -> BTreeSet<String> {
    let mut calls = HashMap::<String, BTreeSet<String>>::new();
    let mut mutating = BTreeSet::new();

    for function in functions {
        let Some(body) = function.body else {
            mutating.insert(function.name.to_owned());
            continue;
        };

        let mut callees: BTreeSet<_> = body.call_sites().map(|call| call.name.to_owned()).collect();
        let writes = body.basic_blocks.values().any(|block| {
            block.statements.iter().any(|statement| match statement {
                MirStatement::Assign(destination, _) => reference_write(body, destination, records),
                MirStatement::ParallelFor { worker, .. } => {
                    callees.insert(worker.clone());

                    false
                }
                _ => false,
            }) || matches!(&block.terminator,
                Some(MirTerminator::Call { destination, .. }
                    | MirTerminator::TryCall { destination, .. }
                    | MirTerminator::CatchCall { destination, .. }) if reference_write(body, destination, records))
        });

        if writes {
            mutating.insert(function.name.to_owned());
        }

        calls.insert(function.name.to_owned(), callees);
    }

    loop {
        let previous = mutating.clone();

        for (function, callees) in &calls {
            if callees
                .iter()
                .any(|callee| !calls.contains_key(callee) || previous.contains(callee))
            {
                mutating.insert(function.clone());
            }
        }

        if previous == mutating {
            return mutating;
        }
    }
}

/// Replace every origin alternative; dropping one can hide a local borrow.
pub(super) fn invalidate(state: &mut State) {
    for value in state.values_mut() {
        for origins in value.0.values_mut() {
            *origins = BTreeSet::from([Origin::Unknown]);
        }
    }
}
