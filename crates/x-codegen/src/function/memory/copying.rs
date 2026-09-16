use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_core_mem_fill(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        let callee = call.name.as_str();
        let [destination_argument, value_argument, count_argument] =
            Self::expect_arguments(callee, arguments)?;
        let destination = self.pointer_argument(destination_argument, "mem_fill_dest")?;
        let element_type = self.core_mem_element_type(call, arguments)?;
        let llvm_element_type = self.codegen.resolve_llvm_type(&element_type)?;
        let mut value = self.value_argument(value_argument, &element_type)?;
        value = self.normalize_value_for_store(
            crate::function::numeric::TypedValue {
                value,
                value_type: &element_type,
            },
            &element_type,
            llvm_element_type,
            "mem_fill_value_adjust",
        )?;
        let count = self.integer_argument(count_argument, &Type::USize)?;

        self.counted_loop_with_names(
            count,
            llvm::LoopNames {
                prefix: "mem_fill",
                index_slot: "mem_fill_i",
                index_load: "mem_fill_index",
                next: "mem_fill_next",
            },
            &mut |index| {
                let element_pointer =
                    self.gep(llvm_element_type, destination, index, "mem_fill_elem")?;
                self.store(element_pointer, value)?;

                Ok(())
            },
        )?;

        Ok(None)
    }

    pub(super) fn emit_core_mem_object_copy(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
        operation: CoreMemoryOperation,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        let function = call.name.as_str();
        let [source_argument, destination_argument, count_argument] =
            Self::expect_arguments(function, arguments)?;
        let source = self.pointer_argument(source_argument, "mem_src")?;
        let destination = self.pointer_argument(destination_argument, "mem_dest")?;
        let count = self.integer_argument(count_argument, &Type::USize)?;
        let element_type = self.core_mem_element_type(call, arguments)?;
        let length = self.element_count_to_bytes(count, &element_type, "mem_copy_len")?;

        if is_object_nonoverlapping_copy(operation) {
            self.memcpy(destination, source, length)?;
        } else {
            self.memmove(destination, source, length)?;
        }

        Ok(None)
    }

    pub(super) fn emit_core_mem_byte_copy(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
        operation: CoreMemoryOperation,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        let function = call.name.as_str();
        let [source_argument, destination_argument, count_argument] =
            Self::expect_arguments(function, arguments)?;
        let source = self.pointer_argument(source_argument, "mem_byte_src")?;
        let destination = self.pointer_argument(destination_argument, "mem_byte_dest")?;
        let length = self.integer_argument(count_argument, &Type::USize)?;

        if operation == CoreMemoryOperation::MoveBytes {
            self.memmove(destination, source, length)?;
        } else {
            self.memcpy(destination, source, length)?;
        }

        Ok(None)
    }
}
