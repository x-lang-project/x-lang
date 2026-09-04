pub(super) use predicates::*;
mod predicates;
use std::cmp::Ordering;

use inkwell::{
    IntPredicate,
    values::{BasicValueEnum, IntValue},
};

use super::FunctionCompiler;
use crate::{
    errors::CodeGenerationError,
    operators::{IntegerSignedness, OperatorEmitter},
};
use x_ast::Operator;
use x_ast::Type;
use x_diagnostics::backend_messages;

pub(super) struct TypedValue<'a, 'ctx> {
    pub value: BasicValueEnum<'ctx>,
    pub value_type: &'a Type,
}
