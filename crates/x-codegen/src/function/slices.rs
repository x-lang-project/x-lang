mod descriptors;

use inkwell::IntPredicate;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_ast::intrinsics::{BuiltinCall, BuiltinKind, SliceOperation};
use x_diagnostics::backend_messages;
use x_mir::Operand;

#[derive(Clone)]
struct SliceValue<'ctx> {
    data: PointerValue<'ctx>,
    len: IntValue<'ctx>,
    element: BasicTypeEnum<'ctx>,
    element_ast: Type,
}

struct CopyEndpoint<'ctx> {
    pointer: PointerValue<'ctx>,
    index: IntValue<'ctx>,
}
