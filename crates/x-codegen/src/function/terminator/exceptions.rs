use inkwell::values::{self, BasicValueEnum};

use super::super::FunctionCompiler;
use super::CallContinuation;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn compile_catch_call_terminator(
        &self,
        callee: &str,
        arguments: &[Operand],
        fallback: &Operand,
        continuation: CallContinuation<'_>,
    ) -> Result<(), CodeGenerationError> {
        let CallContinuation {
            destination,
            target,
        } = continuation;

        let result = self
            .emit_call_value(callee, arguments, None)?
            .ok_or_else(|| {
                CodeGenerationError::LlvmInternalError(
                    backend_messages::throwing_call_function_returned_void(callee),
                )
            })?
            .into_struct_value();
        let succeeded = self
            .b()
            .build_extract_value(result, 0, "catch_ok")?
            .into_int_value();
        let payload = self.b().build_extract_value(result, 1, "catch_value")?;
        let destination_type = self.get_place_type(destination);
        let llvm_type = self.codegen.resolve_llvm_type(&destination_type)?;
        let payload = self.normalize_value_for_store(
            crate::function::numeric::TypedValue {
                value: payload,
                value_type: &self
                    .infer_call_result_type(callee, arguments, &Type::Unknown)
                    .unwrap_or(Type::Unknown),
            },
            &destination_type,
            llvm_type,
            "catch_payload",
        )?;
        let fallback = self.load_operand_value_with_type(fallback, &destination_type)?;
        let selected = self
            .b()
            .build_select(succeeded, payload, fallback, "catch_selected")?;
        let destination_pointer = self.resolve_place_pointer(destination)?;
        self.store(destination_pointer, selected)?;
        let target_block = self.llvm_blocks.get(&target.0).ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::missing_catch_continuation_basic_block_value(target.0),
            )
        })?;
        self.br(*target_block)?;

        Ok(())
    }

    pub(super) fn compile_throw_terminator(
        &self,
        error: &Operand,
    ) -> Result<(), CodeGenerationError> {
        if !self.body.is_throws {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::throw_terminator_escaped_a_non_throws_function(),
            ));
        }

        let return_type = self.return_type_or_void();
        let payload = self.codegen.resolve_llvm_type(return_type)?.const_zero();
        let error = self.load_operand_value_with_type(error, &Type::String)?;
        let result = self.throws_result(false, payload, error)?;
        self.return_value(Some(&result))?;

        Ok(())
    }

    pub(super) fn compile_try_call_terminator(
        &self,
        callee: &str,
        arguments: &[Operand],
        continuation: CallContinuation<'_>,
    ) -> Result<(), CodeGenerationError> {
        let CallContinuation {
            destination,
            target,
        } = continuation;

        if !self.body.is_throws {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::checked_call_function_escaped_a_non_throws_function(callee),
            ));
        }

        let result = self
            .emit_call_value(callee, arguments, None)?
            .ok_or_else(|| {
                CodeGenerationError::LlvmInternalError(
                    backend_messages::throwing_call_function_returned_void(callee),
                )
            })?
            .into_struct_value();
        let succeeded = self
            .b()
            .build_extract_value(result, 0, "throws_ok")?
            .into_int_value();
        let payload = self.b().build_extract_value(result, 1, "throws_value")?;
        let error = self.b().build_extract_value(result, 2, "throws_error")?;

        let function = self.current_function()?;
        let success_block = self
            .context()
            .append_basic_block(function, "throws_success");
        let error_block = self
            .context()
            .append_basic_block(function, "throws_propagate");

        self.condition_br(succeeded, success_block, error_block)?;

        self.b().position_at_end(success_block);
        let destination_type = self.get_place_type(destination);

        if destination_type != Type::Void {
            let destination_pointer = self.resolve_place_pointer(destination)?;
            let llvm_type = self.codegen.resolve_llvm_type(&destination_type)?;
            let payload = self.normalize_value_for_store(
                crate::function::numeric::TypedValue {
                    value: payload,
                    value_type: &self
                        .infer_call_result_type(callee, arguments, &Type::Unknown)
                        .unwrap_or(Type::Unknown),
                },
                &destination_type,
                llvm_type,
                "throws_payload",
            )?;
            self.store(destination_pointer, payload)?;
        }

        let target_block = self.llvm_blocks.get(&target.0).ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::missing_throws_continuation_basic_block_value(target.0),
            )
        })?;
        self.br(*target_block)?;

        self.b().position_at_end(error_block);
        let caller_return_type = self.return_type_or_void();
        let empty_payload = self
            .codegen
            .resolve_llvm_type(caller_return_type)?
            .const_zero();
        let propagated = self.throws_result(false, empty_payload, error)?;
        self.return_value(Some(&propagated))?;

        Ok(())
    }

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
