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

fn local_path(body: &MirBody, place: &Place) -> Option<(LocalIdentifier, Vec<Step>)> {
    match place {
        Place::Local(identifier) => Some((*identifier, Vec::new())),
        Place::Dereference { base, .. } => {
            let (identifier, mut path) = local_path(body, base)?;
            path.push(Step::Dereference);

            Some((identifier, path))
        }
        Place::Field {
            base, field_name, ..
        } => {
            let (identifier, mut path) = local_path(body, base)?;

            if body.place_type(base).pointee().is_some() {
                path.push(Step::Dereference);
            }

            path.push(Step::Field(field_name.clone()));

            Some((identifier, path))
        }
        Place::Index {
            base,
            index: selected,
            ..
        } => {
            let (identifier, mut path) = local_path(body, base)?;

            if body.place_type(base).pointee().is_some() {
                path.push(Step::Dereference);
            }

            path.push(index(selected));

            Some((identifier, path))
        }
        Place::Global { .. } => None,
    }
}

pub(super) fn canonical_value(
    value_type: &x_ast::Type,
    value: &Value,
    records: &Records<'_>,
) -> Value {
    // Bound origin paths by the destination type to keep the analysis finite.
    let allowed = super::model::type_paths(value_type, records);

    Value(
        allowed
            .into_iter()
            .map(|path| {
                let mut origins = value.select(&path).root();

                if origins.is_empty() || path.contains(&Step::Unsupported) {
                    origins.insert(Origin::Unknown);
                }

                let origins = origins
                    .into_iter()
                    .map(|origin| match origin {
                        Origin::Local { ref path, .. } | Origin::Parameter { ref path, .. }
                            if path.len() >= 64 =>
                        {
                            Origin::Unknown
                        }
                        other => other,
                    })
                    .collect();

                (path, origins)
            })
            .collect(),
    )
}

pub(super) fn write(analysis: &Analysis<'_, '_>, place: &Place, value: &Value, state: &mut State) {
    let body = analysis.body;
    let records = analysis.records;
    let value = canonical_value(&body.place_type(place), value, records);
    let value = &value;
    let origins = storage(body, place, state, records);

    if super::havoc::raw_place(body, place)
        || origins.is_empty()
        || origins
            .iter()
            .any(|origin| matches!(origin, Origin::Unknown))
        || (!value.0.is_empty()
            && origins
                .iter()
                .any(|origin| !matches!(origin, Origin::Local { .. })))
    {
        super::havoc::invalidate(state);
    }

    let weak = origins.len() != 1;
    let mut touched = BTreeSet::new();

    for origin in origins {
        if let Origin::Local { identifier, path } = origin {
            let identifier = LocalIdentifier(identifier);

            if !state.contains_key(&identifier) && value.0.is_empty() {
                continue;
            }

            state.entry(identifier).or_default().write(
                &path,
                value,
                weak || path.contains(&Step::Index(None)),
            );
            touched.insert(identifier.0);
        }
    }

    if let Some((identifier, path)) = local_path(body, place)
        && (state.contains_key(&identifier) || !value.0.is_empty())
    {
        state
            .entry(identifier)
            .or_default()
            .write(&path, value, path.contains(&Step::Index(None)));
        touched.insert(identifier.0);
    }

    for identifier in touched {
        let identifier = LocalIdentifier(identifier);

        if let Some(local) = body.locals.get(&identifier)
            && let Some(value) = state.get_mut(&identifier)
        {
            *value = canonical_value(&local.value_type, value, records);

            if value.0.is_empty() {
                state.remove(&identifier);
            }
        }
    }
}
