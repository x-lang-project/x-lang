use super::*;

impl FunctionCompiler<'_, '_> {
    pub(super) fn compile_call_assignment(
        &mut self,
        callee: &str,
        arguments: &[Operand],
        place: &Place,
    ) -> Result<(), CodeGenerationError> {
        let original_place_type = self.get_place_type(place);
        let expected_ast_type = original_place_type.clone();
        let call_result = self.emit_call_value(callee, arguments, Some(&expected_ast_type))?;

        match call_result {
            Some(mut output_value) => {
                if matches!(expected_ast_type, Type::Void) {
                    return Ok(());
                }

                let inferred_type =
                    self.infer_call_result_type(callee, arguments, &expected_ast_type);

                if inferred_type.is_none()
                    && matches!(
                        original_place_type,
                        Type::Unknown | Type::Infer(_) | Type::TypeParameter(_)
                    )
                {
                    return Err(CodeGenerationError::InvalidMir(
                        backend_messages::call_requires_resolved_return_type(callee),
                    ));
                }

                let expected_ast_type =
                    inferred_type.as_ref().unwrap_or(&expected_ast_type).clone();

                if should_reallocate_call_target(place, &original_place_type)
                    && let Place::Local(identifier) = place
                {
                    let llvm_type = if let Some(inferred_type) = inferred_type.as_ref() {
                        self.codegen.resolve_llvm_type(inferred_type)?
                    } else if is_aggregate_value(output_value) {
                        output_value.get_type()
                    } else {
                        self.codegen.resolve_llvm_type(&expected_ast_type)?
                    };

                    self.replace_local_storage(*identifier, expected_ast_type.clone(), llvm_type)?;
                }

                let target_pointer = self.resolve_place_pointer(place)?;
                let expected_llvm_type = self.codegen.resolve_llvm_type(&expected_ast_type)?;

                output_value = self.normalize_value_for_store(
                    crate::function::numeric::TypedValue {
                        value: output_value,
                        value_type: &self
                            .infer_call_result_type(callee, arguments, &Type::Unknown)
                            .unwrap_or(Type::Unknown),
                    },
                    &expected_ast_type,
                    expected_llvm_type,
                    "call_result_store_adjust",
                )?;

                self.store(target_pointer, output_value)?;
                self.emit_enclosing_invariants_after_store(place)?;

                Ok(())
            }

            None => {
                if matches!(
                    expected_ast_type,
                    Type::Void | Type::Unknown | Type::Infer(_)
                ) {
                    return Ok(());
                }

                Err(CodeGenerationError::LlvmInternalError(backend_messages::call_function_returns_void_but_assignment_target_expects_expected_ast_type(callee, expected_ast_type)))
            }
        }
    }
}
