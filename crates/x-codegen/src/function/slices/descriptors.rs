use inkwell::values::{self, BasicValueEnum, PointerValue};

use super::super::FunctionCompiler;

use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn current_function(
        &self,
    ) -> Result<values::FunctionValue<'ctx>, CodeGenerationError> {
        self.b()
            .get_insert_block()
            .and_then(|block| block.get_parent())
            .ok_or_else(|| {
                CodeGenerationError::LlvmInternalError(
                    backend_messages::missing_current_llvm_function(),
                )
            })
    }
}
