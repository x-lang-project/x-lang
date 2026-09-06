use inkwell::values;
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use super::llvm;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_ast::intrinsics::{BuiltinCall, BuiltinKind, CoreMemoryOperation};
use x_mir::Operand;

fn is_object_nonoverlapping_copy(operation: CoreMemoryOperation) -> bool {
    operation == CoreMemoryOperation::CopyObjectsNonoverlapping
}
