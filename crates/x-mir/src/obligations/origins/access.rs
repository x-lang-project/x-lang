use std::collections::BTreeSet;

use super::{
    Analysis, Records,
    model::{Origin, State, Step, Value, unknown},
};
use crate::{LocalIdentifier, MirBody, Operand, Place};

fn index(value: &Operand) -> Step {
    Step::Index(if let Operand::ConstantInteger(index) = value {
        Some(*index)
    } else {
        None
    })
}

pub(super) fn dereference(value: &Value, state: &State) -> Value {
    let mut result = value.select(&[Step::Dereference]);

    for origin in value.root() {
        if let Origin::Local { identifier, path } = origin
            && let Some(local) = state.get(&LocalIdentifier(identifier))
        {
            result.join(&local.select(&path));
        }
    }

    result
}

pub(super) fn read(body: &MirBody, place: &Place, state: &State, records: &Records<'_>) -> Value {
    match place {
        Place::Local(identifier) => state
            .get(identifier)
            .cloned()
            .unwrap_or_else(|| unknown(&body.place_type(place), records)),
        Place::Global { value_type, .. } => unknown(value_type, records),
        Place::Dereference { base, .. } => dereference(&read(body, base, state, records), state),
        Place::Field {
            base, field_name, ..
        } => {
            let value = read(body, base, state, records);
            let value = if body.place_type(base).pointee().is_some() {
                dereference(&value, state)
            } else {
                value
            };

            value.select(&[Step::Field(field_name.clone())])
        }
        Place::Index {
            base,
            index: selected,
            ..
        } => {
            let value = read(body, base, state, records);
            let value = if body.place_type(base).pointee().is_some() {
                dereference(&value, state)
            } else {
                value
            };

            value.select(&[index(selected)])
        }
    }
}

fn projected(origins: BTreeSet<Origin>, step: Step) -> BTreeSet<Origin> {
    origins
        .into_iter()
        .map(|origin| {
            if let Origin::Local {
                identifier,
                mut path,
            } = origin
            {
                path.push(step.clone());

                Origin::Local { identifier, path }
            } else {
                origin
            }
        })
        .collect()
}

pub(super) fn storage(
    body: &MirBody,
    place: &Place,
    state: &State,
    records: &Records<'_>,
) -> BTreeSet<Origin> {
    match place {
        Place::Local(identifier) => BTreeSet::from([Origin::Local {
            identifier: identifier.0,
            path: Vec::new(),
        }]),
        Place::Global { name, .. } => BTreeSet::from([Origin::Global(name.clone())]),
        Place::Dereference { base, .. } => read(body, base, state, records).root(),
        Place::Field {
            base, field_name, ..
        } => {
            let origins = if body.place_type(base).pointee().is_some() {
                read(body, base, state, records).root()
            } else {
                storage(body, base, state, records)
            };

            projected(origins, Step::Field(field_name.clone()))
        }
        Place::Index {
            base,
            index: selected,
            ..
        } => {
            let origins = if body.place_type(base).pointee().is_some() {
                read(body, base, state, records).root()
            } else {
                storage(body, base, state, records)
            };

            projected(origins, index(selected))
        }
    }
}
