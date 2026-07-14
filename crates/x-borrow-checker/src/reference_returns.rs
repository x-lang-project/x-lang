use crate::ReferenceProjection;
use traversal::{LifetimeTraversal, OriginRequirement};
use x_ast::{Expression, ExpressionKind, Lifetime, Type};
use x_diagnostics::borrow_messages;

mod traversal;
