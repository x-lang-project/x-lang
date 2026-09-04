mod casts;
use inkwell::IntPredicate;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use super::numeric;
use super::numeric::is_signed_integer_type;
use crate::errors::CodeGenerationError;
use crate::function_helpers::*;
use x_ast::{Operator, Type, UnaryOperator};
use x_diagnostics::backend_messages;
use x_mir::{Operand, Place, Rvalue};

fn should_coerce_integer_operand(
    operand: &Operand,
    operand_type: Option<&Type>,
    peer_type: &Type,
) -> bool {
    let needs_concrete_type = matches!(operand, Operand::ConstantInteger(_))
        || operand_type.is_some_and(is_generic_placeholder_type)
        || operand_type
            .is_some_and(|value_type| matches!(value_type, Type::Unknown | Type::Infer(_)));

    needs_concrete_type && is_integer_ast_type(peer_type)
}
