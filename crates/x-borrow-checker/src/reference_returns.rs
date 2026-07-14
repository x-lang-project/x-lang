use super::parameter_origins;
use crate::ReferenceProjection;
use traversal::{LifetimeTraversal, OriginRequirement};
use x_ast::{Expression, ExpressionKind, Lifetime, Type};
use x_diagnostics::borrow_messages;

mod traversal;

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
