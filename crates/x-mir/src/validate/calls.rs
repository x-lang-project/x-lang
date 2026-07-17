use super::MirValidationError;
use crate::{MirBody, Place, ResolvedCall};
use x_ast::intrinsics;

pub(super) fn validate_call_identities(
    function: &str,
    body: &MirBody,
) -> Result<(), MirValidationError> {
    for (name, target) in &body.call_targets {
        let consistent = match target {
            ResolvedCall::Source { name: source, .. } => source == name,
            ResolvedCall::Builtin(signatures) => {
                !signatures.is_empty()
                    && signatures.iter().all(|identity| {
                        Some(identity.clone())
                            == intrinsics::BuiltinCall::resolve(
                                name,
                                &identity.argument_types,
                                identity.return_type.as_ref(),
                            )
                    })
            }
        };

        if !consistent {
            return Err(call_identity_error(function, name));
        }
    }

    Ok(())
}

fn call_identity_error(function: &str, call: &str) -> MirValidationError {
    MirValidationError::CallIdentityMismatch {
        function: function.into(),
        call: call.into(),
    }
}

// Lifetime names may differ after call instantiation; pointee types must match.
fn same_runtime_type(declared: &x_ast::Type, actual: &x_ast::Type) -> bool {
    use x_ast::Type;

    match (declared, actual) {
        (
            Type::Reference {
                is_mut: first_value,
                is_unique: u,
                inner: element,
                ..
            },
            Type::Reference {
                is_mut: second_value,
                is_unique: value,
                inner: other_value,
                ..
            },
        ) => first_value == second_value && u == value && same_runtime_type(element, other_value),
        (
            Type::RawPointer {
                is_mut: first_value,
                inner: element,
            },
            Type::RawPointer {
                is_mut: second_value,
                inner: other_value,
            },
        ) => first_value == second_value && same_runtime_type(element, other_value),
        (Type::Array(element), Type::Array(other_value)) => same_runtime_type(element, other_value),
        (
            Type::FixedSizedArray(element, first_value),
            Type::FixedSizedArray(other_value, second_value),
        ) => first_value == second_value && same_runtime_type(element, other_value),
        (Type::Tuple(element), Type::Tuple(other_value)) => {
            same_runtime_items(element, other_value)
        }
        (
            Type::GenericInstance {
                name: first_value,
                type_arguments: element,
            },
            Type::GenericInstance {
                name: second_value,
                type_arguments: other_value,
            },
        ) => first_value == second_value && same_runtime_items(element, other_value),
        (
            Type::Function {
                parameters: element,
                return_value: first_value,
            },
            Type::Function {
                parameters: other_value,
                return_value: second_value,
            },
        ) => {
            same_runtime_items(element, other_value) && same_runtime_type(first_value, second_value)
        }
        _ => declared == actual,
    }
}

fn same_runtime_items(declared: &[x_ast::Type], actual: &[x_ast::Type]) -> bool {
    declared.len() == actual.len()
        && declared
            .iter()
            .zip(actual)
            .all(|(first_value, second_value)| same_runtime_type(first_value, second_value))
}
