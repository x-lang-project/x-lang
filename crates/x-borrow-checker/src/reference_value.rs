use std::collections::{BTreeSet, HashMap};

use crate::ReferenceTarget;

mod evaluation;
mod loans;
mod storage;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReferenceProjection {
    Field(String),
    Index(i64),
    AnyIndex,
}

impl ReferenceProjection {
    pub(crate) fn from_field_type(value_type: &x_ast::Type, name: &str) -> Self {
        let mut value_type = value_type;

        while let x_ast::Type::Reference { inner, .. } = value_type {
            value_type = inner;
        }

        if matches!(value_type, x_ast::Type::Tuple(_))
            && let Ok(index) = name.parse()
        {
            return Self::Index(index);
        }

        Self::Field(name.into())
    }

    pub(crate) fn from_field_path(
        value_type: &x_ast::Type,
        path: &str,
        layouts: &HashMap<String, x_ast::StructDefinition>,
    ) -> Vec<Self> {
        let mut value_type = value_type.clone();

        path.split('.')
            .map(|name| {
                while let x_ast::Type::Reference { inner, .. } = &value_type {
                    value_type = (**inner).clone();
                }

                let projection = Self::from_field_type(&value_type, name);
                value_type = match (&value_type, &projection) {
                    (x_ast::Type::Tuple(items), Self::Index(index)) => usize::try_from(*index)
                        .ok()
                        .and_then(|index| items.get(index))
                        .cloned()
                        .unwrap_or(x_ast::Type::Unknown),
                    _ => crate::parameter_origins::fields(&value_type, layouts)
                        .and_then(|fields| {
                            fields
                                .into_iter()
                                .find(|(field, _)| field == name)
                                .map(|(_, value_type)| value_type)
                        })
                        .unwrap_or(x_ast::Type::Unknown),
                };

                projection
            })
            .collect()
    }

    pub(crate) fn from_index_expression(expression: &x_ast::Expression) -> Self {
        match expression.kind {
            x_ast::ExpressionKind::Integer(index) => Self::Index(index),
            _ => Self::AnyIndex,
        }
    }
}

impl std::fmt::Display for ReferenceProjection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(name) => formatter.write_str(name),
            Self::Index(index) => index.fmt(formatter),
            Self::AnyIndex => formatter.write_str("*"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReferenceValue {
    pub targets: Vec<ReferenceTarget>,
    pub origin_known: bool,
    pub fields: HashMap<ReferenceProjection, ReferenceValue>,
    pub is_unreachable: bool,
}

impl ReferenceValue {
    fn shares_owner(&self, other: &Self) -> bool {
        self.find_target(|stored| {
            other
                .find_target(|returned| {
                    stored.origin_region == returned.origin_region
                        && stored.binding_generation == returned.binding_generation
                        && stored.target_variable == returned.target_variable
                })
                .is_some()
        })
        .is_some()
    }

    pub(super) fn project(&self, path: &[ReferenceProjection]) -> Self {
        if self.is_unreachable {
            return self.clone();
        }

        let Some((head, tail)) = path.split_first() else {
            return self.clone();
        };

        let selected = if *head == ReferenceProjection::AnyIndex {
            self.fields
                .iter()
                .filter(|(key, _)| {
                    matches!(
                        key,
                        ReferenceProjection::Index(_) | ReferenceProjection::AnyIndex
                    )
                })
                .map(|(_, value)| value.clone())
                .reduce(|first_value, second_value| first_value.join(&second_value))
                .unwrap_or_default()
        } else {
            let exact = self.fields.get(head);

            let wildcard = matches!(head, ReferenceProjection::Index(_))
                .then(|| self.fields.get(&ReferenceProjection::AnyIndex))
                .flatten();

            match (exact, wildcard) {
                (Some(value), Some(wildcard)) => value.join(wildcard),
                (Some(value), None) | (None, Some(value)) => value.clone(),
                (None, None) => Self::default(),
            }
        };

        selected.project(tail)
    }

    pub(super) fn join(&self, other: &Self) -> Self {
        if self.is_unreachable {
            return other.clone();
        }

        if other.is_unreachable {
            return self.clone();
        }

        let mut targets = self.targets.clone();
        targets.extend(other.targets.iter().cloned());
        // Fixed-point equality must not depend on branch or map iteration order.
        targets.sort_unstable();
        targets.dedup();
        let unknown = Self::default();
        let fields = self
            .fields
            .keys()
            .chain(other.fields.keys())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|key| {
                let left = self.fields.get(key).unwrap_or(&unknown);
                let right = other.fields.get(key).unwrap_or(&unknown);

                (key.clone(), left.join(right))
            })
            .collect();

        Self {
            targets,
            origin_known: self.origin_known && other.origin_known,
            fields,
            is_unreachable: false,
        }
    }

    pub(super) fn find_target(
        &self,
        predicate: impl Fn(&ReferenceTarget) -> bool + Copy,
    ) -> Option<&ReferenceTarget> {
        self.targets
            .iter()
            .find(|target| predicate(target))
            .or_else(|| {
                self.fields
                    .values()
                    .find_map(|value| value.find_target(predicate))
            })
    }

    pub(super) fn assign(&mut self, path: &[ReferenceProjection], value: Self) {
        let Some((head, tail)) = path.split_first() else {
            *self = value;
            return;
        };

        if *head == ReferenceProjection::AnyIndex {
            let mut assigned = Self::default();
            assigned.assign(tail, value);

            if let Some(previous) = self.fields.get_mut(head) {
                *previous = previous.join(&assigned);
            } else {
                self.fields.insert(head.clone(), assigned);
            }
        } else {
            self.fields
                .entry(head.clone())
                .or_default()
                .assign(tail, value);
        }
    }
}
