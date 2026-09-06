use inkwell::values;
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use super::llvm;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_ast::intrinsics::{BuiltinCall, BuiltinKind, CoreMemoryOperation};
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    fn core_mem_element_type(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<Type, CodeGenerationError> {
        self.pointer_memory_element_type(call, arguments, 0)
    }

    fn ast_type_size_bytes(&self, value_type: &Type) -> Result<u64, CodeGenerationError> {
        self.codegen
            .resolve_type_layout(value_type)
            .map(|(size, _)| size)
    }
}

fn is_object_nonoverlapping_copy(operation: CoreMemoryOperation) -> bool {
    operation == CoreMemoryOperation::CopyObjectsNonoverlapping
}
