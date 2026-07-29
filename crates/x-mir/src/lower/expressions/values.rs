use super::*;
use crate::{MirStatement, Place, Rvalue};
use x_diagnostics::backend_messages;

pub(super) fn unsupported_shape(shape: ExpressionKind) -> MirLowerError {
    MirLowerError::UnsupportedExpression(
        backend_messages::control_flow_evaluation_is_not_implemented_inside_shape(shape),
    )
}
