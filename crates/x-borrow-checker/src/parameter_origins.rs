use std::collections::HashMap;

use crate::ReferenceProjection;
use x_ast::{StructDefinition, Type};
use x_diagnostics::borrow_messages;

struct ParameterOrigins<'a> {
    layouts: &'a HashMap<String, StructDefinition>,
    origins: &'a mut HashMap<String, Type>,
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
