use std::borrow::Cow;

use inkwell::module;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values;

use super::FunctionCompiler;
use super::constants::BlockConstants;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::{MirStatement, Operand, Place, Rvalue};

struct AssignmentTarget<'place, 'ctx> {
    place: Cow<'place, Place>,
    pointer: values::PointerValue<'ctx>,
    ast_type: Type,
    llvm_type: BasicTypeEnum<'ctx>,
}

fn should_reallocate_call_target(place: &Place, original_place_type: &Type) -> bool {
    should_specialize_assignment_target(place, original_place_type)
}

fn has_incompatible_struct_result(expected_is_struct: bool, output_is_struct: bool) -> bool {
    expected_is_struct && !output_is_struct
}

fn is_aggregate_value(value: values::BasicValueEnum<'_>) -> bool {
    value.is_struct_value() || value.is_array_value()
}

fn should_specialize_assignment_target(place: &Place, expected_type: &Type) -> bool {
    matches!(place, Place::Local(_)) && matches!(expected_type, Type::Infer(_) | Type::Unknown)
}
