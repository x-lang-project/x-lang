mod buffered_calls;

use super::MirLowerer;
use crate::errors::MirLowerError;

use crate::{LocalIdentifier, MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::backend_messages;
