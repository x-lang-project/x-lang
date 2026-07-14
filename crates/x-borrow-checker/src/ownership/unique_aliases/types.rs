use std::collections::HashMap;

use crate::{ReferenceProjection, ReferenceValue, parameter_origins};
use x_ast::{StructDefinition, Type};

pub(super) fn stored_children(
    value_type: &Type,
    value: &ReferenceValue,
    layouts: &HashMap<String, StructDefinition>,
) -> Option<Vec<(ReferenceProjection, Type)>> {
    match value_type {
        Type::Custom(_) | Type::GenericInstance { .. } => Some(
            parameter_origins::fields(value_type, layouts)?
                .into_iter()
                .map(|(field, value_type)| (ReferenceProjection::Field(field), value_type))
                .collect(),
        ),
        Type::Tuple(items) => Some(
            items
                .iter()
                .enumerate()
                .map(|(index, value_type)| {
                    (ReferenceProjection::Index(index as i64), value_type.clone())
                })
                .collect(),
        ),
        Type::FixedSizedArray(_, 0) => Some(Vec::new()),
        Type::FixedSizedArray(inner, _) | Type::Array(inner) => {
            let mut children = value
                .fields
                .keys()
                .map(|field| (field.clone(), inner.as_ref().clone()))
                .collect::<Vec<_>>();
            let covered = value.fields.contains_key(&ReferenceProjection::AnyIndex)
                || matches!(value_type, Type::FixedSizedArray(_, length) if value.fields.len() == *length);

            if !covered {
                children.push((ReferenceProjection::AnyIndex, inner.as_ref().clone()));
            }

            Some(children)
        }
        Type::Unknown | Type::Infer(_) | Type::TypeParameter(_) => None,
        _ => Some(Vec::new()),
    }
}
