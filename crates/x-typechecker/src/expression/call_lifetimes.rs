use std::collections::HashMap;

use crate::{FunctionSignature, errors::TypeError};
use x_ast::{Lifetime, Type};

type Bindings = HashMap<String, Option<Lifetime>>;
#[derive(Default)]
struct CallBindings {
    lifetimes: Bindings,
    invariant: bool,
    pins: Vec<(String, Option<Lifetime>)>,
}

pub(super) fn instantiate(
    signature: &FunctionSignature,
    actual: &[Type],
) -> Result<FunctionSignature, TypeError> {
    let mut bindings = CallBindings::default();

    for (expected, actual) in signature.parameter_types.iter().zip(actual) {
        collect(expected, actual, &mut bindings);
    }

    for (name, pinned) in &bindings.pins {
        if bindings.lifetimes.get(name) != Some(pinned) {
            return Err(crate::type_mismatch!(
                format!("invariant reference lifetime {pinned:?}"),
                format!(
                    "common argument lifetime {:?}",
                    bindings.lifetimes.get(name)
                )
            ));
        }
    }

    let mut instantiated = signature.clone();

    for parameter in &mut instantiated.parameter_types {
        substitute(parameter, &bindings.lifetimes);
    }

    substitute(&mut instantiated.return_type, &bindings.lifetimes);

    Ok(instantiated)
}

fn collect(expected: &Type, actual: &Type, bindings: &mut CallBindings) {
    match (expected, actual) {
        (
            Type::Reference {
                lifetime, inner, ..
            },
            actual,
        ) => {
            let (actual_lifetime, actual_inner) = match actual {
                Type::Reference {
                    lifetime, inner, ..
                } => (lifetime.clone(), inner.as_ref()),
                _ => (None, actual),
            };

            if let Some(Lifetime::Named(name)) = lifetime {
                if bindings.invariant {
                    bindings.pins.push((name.clone(), actual_lifetime.clone()));
                }

                bindings
                    .lifetimes
                    .entry(name.clone())
                    .and_modify(|previous| {
                        if *previous == actual_lifetime {
                            return;
                        }

                        if *previous == Some(Lifetime::Static) {
                            *previous = actual_lifetime.clone();
                        } else if actual_lifetime != Some(Lifetime::Static) {
                            // Differing origins share an unnamed lifetime; borrow checking validates escapes.
                            *previous = None;
                        }
                    })
                    .or_insert(actual_lifetime);
            }

            let invariant = bindings.invariant;
            bindings.invariant |= matches!(expected, Type::Reference { is_mut: true, .. });
            collect(inner, actual_inner, bindings);
            bindings.invariant = invariant;
        }
        (
            Type::RawPointer {
                inner: expected, ..
            },
            Type::RawPointer { inner: actual, .. },
        )
        | (Type::Array(expected), Type::Array(actual))
        | (Type::FixedSizedArray(expected, _), Type::FixedSizedArray(actual, _))
        | (Type::Array(expected), Type::FixedSizedArray(actual, _))
        | (
            Type::Simd {
                element: expected, ..
            },
            Type::Simd {
                element: actual, ..
            },
        ) => collect(expected, actual, bindings),
        (Type::Tuple(expected), Type::Tuple(actual))
        | (
            Type::GenericInstance {
                type_arguments: expected,
                ..
            },
            Type::GenericInstance {
                type_arguments: actual,
                ..
            },
        ) => {
            for (expected, actual) in expected.iter().zip(actual) {
                collect(expected, actual, bindings);
            }
        }
        (
            Type::Function {
                parameters: expected,
                return_value: expected_return,
            },
            Type::Function {
                parameters: actual,
                return_value: actual_return,
            },
        ) => {
            for (expected, actual) in expected.iter().zip(actual) {
                collect(expected, actual, bindings);
            }

            collect(expected_return, actual_return, bindings);
        }
        _ => {}
    }
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
