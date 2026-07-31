use super::ExpressionLowerer;
use super::helpers::method_call_function_name;
use crate::errors::MirLowerError;
use crate::{MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, Type, UnaryOperator};
use x_diagnostics::backend_messages;
use x_diagnostics::mir_messages;

mod preparation;
pub(crate) use preparation::PreparedCall;
