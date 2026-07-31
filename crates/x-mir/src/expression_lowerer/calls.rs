use super::ExpressionLowerer;
use super::helpers::method_call_function_name;
use crate::errors::MirLowerError;
use crate::{MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, Type, UnaryOperator};
use x_diagnostics::backend_messages;
use x_diagnostics::mir_messages;

mod preparation;
pub(crate) use preparation::PreparedCall;

fn nominal_receiver(value_type: &Type) -> bool {
    match value_type {
        Type::Custom(_) | Type::GenericInstance { .. } => true,
        Type::Reference { inner, .. } => nominal_receiver(inner),
        _ => false,
    }
}
