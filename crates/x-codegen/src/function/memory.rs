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

    pub(super) fn element_count_to_bytes(
        &self,
        count: values::IntValue<'ctx>,
        element_type: &Type,
        name: &str,
    ) -> Result<values::IntValue<'ctx>, CodeGenerationError> {
        let element_size = self.ast_type_size_bytes(element_type)?;
        let size_type = self
            .codegen
            .resolve_llvm_type(&Type::USize)?
            .into_int_type();
        let count = if count.get_type() == size_type {
            count
        } else {
            self.integer_cast(count, size_type, &format!("{name}_count"))?
        };

        if element_size == 1 {
            return Ok(count);
        }

        self.integer_mul(count, size_type.const_int(element_size, false), name)
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
