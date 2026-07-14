use super::parameter_origins;
use x_ast::{GenericParameter, Lifetime, Type};

pub(super) fn has_non_lifetime_generic_parameter(parameters: &[GenericParameter]) -> bool {
    parameters
        .iter()
        .any(|parameter| !matches!(parameter, GenericParameter::Lifetime { .. }))
}

pub(super) fn declared_reference_lifetime(value_type: Option<&Type>) -> Option<Lifetime> {
    match value_type {
        Some(Type::Reference { lifetime, .. }) => lifetime.clone(),
        _ => None,
    }
}
