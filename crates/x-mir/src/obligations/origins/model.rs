use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::Records;
use crate::LocalIdentifier;
use x_ast::Type;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::obligations) enum Step {
    Field(String),
    Index(Option<i64>),
    Dereference,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::obligations) enum Origin {
    Parameter { index: usize, path: Vec<Step> },
    Local { identifier: usize, path: Vec<Step> },
    Global(String),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(in crate::obligations) struct Value(pub BTreeMap<Vec<Step>, BTreeSet<Origin>>);

fn matches_step(first_value: &Step, second_value: &Step) -> bool {
    first_value == second_value
        || matches!(
            (first_value, second_value),
            (Step::Index(None), Step::Index(_)) | (Step::Index(_), Step::Index(None))
        )
}

fn starts(path: &[Step], prefix: &[Step]) -> bool {
    path.len() >= prefix.len()
        && path
            .iter()
            .zip(prefix)
            .all(|(first_value, second_value)| matches_step(first_value, second_value))
}

impl Value {
    pub fn root(&self) -> BTreeSet<Origin> {
        self.0.get(&Vec::new()).cloned().unwrap_or_default()
    }

    pub fn select(&self, prefix: &[Step]) -> Self {
        let mut output = Self::default();

        for (path, origins) in &self.0 {
            if starts(path, prefix) {
                output
                    .0
                    .entry(path.iter().skip(prefix.len()).cloned().collect())
                    .or_default()
                    .extend(origins.clone());
            }
        }

        output
    }

    pub fn prefixed(&self, prefix: &[Step]) -> Self {
        Self(
            self.0
                .iter()
                .map(|(path, origins)| {
                    (
                        prefix.iter().chain(path).cloned().collect(),
                        origins.clone(),
                    )
                })
                .collect(),
        )
    }

    pub fn join(&mut self, other: &Self) {
        for (path, origins) in &other.0 {
            self.0
                .entry(path.clone())
                .or_default()
                .extend(origins.clone());
        }
    }

    pub fn write(&mut self, prefix: &[Step], value: &Self, weak: bool) {
        if !weak {
            self.0.retain(|path, _| !starts(path, prefix));
        }

        self.join(&value.prefixed(prefix));
    }
}

pub(super) type State = HashMap<LocalIdentifier, Value>;

pub(super) fn type_paths(value_type: &Type, records: &Records<'_>) -> Vec<Vec<Step>> {
    struct Paths<'a, 'r> {
        records: &'a Records<'r>,
        output: Vec<Vec<Step>>,
    }
    impl Paths<'_, '_> {
        fn visit(&mut self, value_type: &Type, path: Vec<Step>, depth: usize) {
            if depth >= 64 {
                self.unsupported(path);
                return;
            }

            match value_type {
                Type::Reference { inner, .. } | Type::RawPointer { inner, .. } => {
                    self.output.push(path.clone());
                    let mut nested = path;
                    nested.push(Step::Dereference);
                    self.visit(inner, nested, depth + 1);
                }
                Type::Custom(name) | Type::GenericInstance { name, .. } => {
                    if let Some(record) = self.records.get(name.as_str()) {
                        for (field, value_type) in &record.fields {
                            let mut nested = path.clone();
                            nested.push(Step::Field(field.clone()));
                            self.visit(value_type, nested, depth + 1);
                        }
                    } else {
                        self.unsupported(path);
                    }
                }
                Type::TypeParameter(_) => self.unsupported(path),
                Type::Array(inner) | Type::FixedSizedArray(inner, _) => {
                    let mut nested = path;
                    nested.push(Step::Index(None));
                    self.visit(inner, nested, depth + 1);
                }
                Type::Tuple(fields) => {
                    for (index, value_type) in fields.iter().enumerate() {
                        let mut nested = path.clone();
                        nested.push(Step::Field(index.to_string()));
                        self.visit(value_type, nested, depth + 1);
                    }
                }
                _ => {}
            }
        }

        fn unsupported(&mut self, mut path: Vec<Step>) {
            path.push(Step::Unsupported);
            self.output.push(path);
        }
    }
    let mut paths = Paths {
        records,
        output: Vec::new(),
    };
    paths.visit(value_type, Vec::new(), 0);

    paths.output
}

pub(super) fn unknown(value_type: &Type, records: &Records<'_>) -> Value {
    Value(
        type_paths(value_type, records)
            .into_iter()
            .map(|path| (path, BTreeSet::from([Origin::Unknown])))
            .collect(),
    )
}

pub(super) fn entry(body: &crate::MirBody, records: &Records<'_>) -> State {
    body.locals
        .iter()
        .filter_map(|(identifier, local)| {
            let paths = type_paths(&local.value_type, records);

            if paths.is_empty() {
                return None;
            }

            let value = Value(
                paths
                    .into_iter()
                    .map(|path| {
                        let origin = if !path.contains(&Step::Unsupported)
                            && identifier.0 > 0
                            && identifier.0 <= body.argument_count
                        {
                            Origin::Parameter {
                                index: identifier.0 - 1,
                                path: path.clone(),
                            }
                        } else {
                            Origin::Unknown
                        };

                        (path, BTreeSet::from([origin]))
                    })
                    .collect(),
            );

            Some((*identifier, value))
        })
        .collect()
}
