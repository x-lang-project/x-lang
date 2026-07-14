use super::FunctionLifetimeContext;
use super::ReferenceValue;
use super::Region;
use super::parameter_origins;
use crate::ReferenceProjection;
use traversal::{LifetimeTraversal, OriginRequirement};
use x_ast::{Expression, ExpressionKind, Lifetime, Type};
use x_diagnostics::borrow_messages;

mod traversal;

fn parameter_origins_satisfy(
    value: &ReferenceValue,
    context: &FunctionLifetimeContext,
    expected: &Lifetime,
) -> bool {
    value.origin_known
        && !value.targets.is_empty()
        && value.targets.iter().all(|target| {
            target.origin_region == Region(0)
                && context
                    .parameter_references
                    .get(&target.target_variable)
                    .is_some_and(|value_type| {
                        type_reference_lifetime_satisfies(value_type, expected)
                    })
        })
}

fn type_reference_lifetime_satisfies(value_type: &Type, expected: &Lifetime) -> bool {
    match value_type {
        Type::Reference { lifetime, .. } => match (lifetime, expected) {
            (Some(Lifetime::Static), _) => true,
            (Some(Lifetime::Named(actual)), Lifetime::Named(expected)) => actual == expected,
            (Some(Lifetime::Erased), _) | (Some(Lifetime::Infer(_)), _) => true,
            (None, Lifetime::Erased | Lifetime::Infer(_)) => true,
            _ => false,
        },
        _ => false,
    }
}
