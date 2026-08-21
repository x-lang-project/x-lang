use std::collections::HashMap;

use crate::FunctionSignature;
use x_ast::{Lifetime, Type};

type Bindings = HashMap<String, Option<Lifetime>>;
#[derive(Default)]
struct CallBindings {
    lifetimes: Bindings,
    invariant: bool,
    pins: Vec<(String, Option<Lifetime>)>,
}

fn substitute(value_type: &mut Type, bindings: &Bindings) {
    match value_type {
        Type::Reference {
            lifetime, inner, ..
        } => {
            if let Some(Lifetime::Named(name)) = lifetime
                && let Some(bound) = bindings.get(name)
            {
                *lifetime = bound.clone();
            }

            substitute(inner, bindings);
        }
        Type::RawPointer { inner, .. }
        | Type::Array(inner)
        | Type::FixedSizedArray(inner, _)
        | Type::Simd { element: inner, .. } => substitute(inner, bindings),
        Type::Tuple(items)
        | Type::GenericInstance {
            type_arguments: items,
            ..
        } => {
            for item in items {
                substitute(item, bindings);
            }
        }
        Type::Function {
            parameters,
            return_value,
        } => {
            for parameter in parameters {
                substitute(parameter, bindings);
            }

            substitute(return_value, bindings);
        }
        _ => {}
    }
}
