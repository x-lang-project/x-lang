use inkwell::values::{self, BasicValueEnum};

use super::super::FunctionCompiler;
use super::CallContinuation;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn throws_result(
        &self,
        succeeded: bool,
        payload: BasicValueEnum<'ctx>,
        error: BasicValueEnum<'ctx>,
    ) -> Result<values::StructValue<'ctx>, CodeGenerationError> {
        let result_type = self
            .current_function()?
            .get_type()
            .get_return_type()
            .ok_or_else(|| {
                CodeGenerationError::LlvmInternalError(
                    backend_messages::throws_function_has_no_aggregate_return_type(),
                )
            })?
            .into_struct_type();

        self.struct_value(
            result_type,
            &[
                (
                    self.boolean_type()
                        .const_int(u64::from(succeeded), false)
                        .into(),
                    "throws_tag",
                ),
                (payload, "throws_payload"),
                (error, "throws_error"),
            ],
        )
    }
}
