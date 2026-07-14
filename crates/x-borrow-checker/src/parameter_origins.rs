use std::collections::HashMap;

use super::ReferenceTarget;
use super::ReferenceValue;
use super::Region;
use crate::ReferenceProjection;
use x_ast::{StructDefinition, Type};
use x_diagnostics::borrow_messages;

struct ParameterOrigins<'a> {
    layouts: &'a HashMap<String, StructDefinition>,
    origins: &'a mut HashMap<String, Type>,
}

pub(super) fn fields(
    value_type: &Type,
    layouts: &HashMap<String, StructDefinition>,
) -> Option<Vec<(String, Type)>> {
    let name = match value_type {
        Type::Custom(name) | Type::GenericInstance { name, .. } => name,
        _ => return None,
    };

    let layout = layouts.get(name)?;
    let substitutions = match value_type {
        Type::GenericInstance { type_arguments, .. } => layout
            .generic_parameters
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter_map(x_ast::GenericParameter::non_lifetime_name)
            .cloned()
            .zip(type_arguments.iter().cloned())
            .collect(),
        _ => HashMap::new(),
    };

    Some(
        layout
            .fields
            .iter()
            .map(|(name, value_type)| (name.clone(), substitute(value_type, &substitutions)))
            .collect(),
    )
}

fn substitute(value_type: &Type, substitutions: &HashMap<String, Type>) -> Type {
    let mut value_type = value_type.clone();

    match &mut value_type {
        Type::TypeParameter(name) | Type::Custom(name) if substitutions.contains_key(name) => {
            if let Some(value) = substitutions.get(name) {
                return value.clone();
            }
        }
        Type::Reference { inner, .. }
        | Type::RawPointer { inner, .. }
        | Type::Array(inner)
        | Type::FixedSizedArray(inner, _)
        | Type::Simd { element: inner, .. } => **inner = substitute(inner, substitutions),
        Type::Tuple(items)
        | Type::GenericInstance {
            type_arguments: items,
            ..
        } => {
            for item in items {
                *item = substitute(item, substitutions);
            }
        }
        Type::Function {
            parameters,
            return_value,
        } => {
            for parameter in parameters {
                *parameter = substitute(parameter, substitutions);
            }
            **return_value = substitute(return_value, substitutions);
        }
        _ => {}
    }

    value_type
}
