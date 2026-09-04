mod dependencies;

use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use super::numeric;
use crate::errors::CodeGenerationError;
use x_ast::Operator;
use x_ast::{Expression, ExpressionKind, Type, UnaryOperator};
use x_diagnostics::backend_messages;
use x_mir::Place;
