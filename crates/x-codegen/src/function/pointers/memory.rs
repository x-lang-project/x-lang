use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn emit_pointer_read_intrinsic(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let callee = call.name.as_str();
        let [argument] = self.arguments(callee, arguments).exact()?;

        let pointer = argument.pointer("ptr_read_addr")?;
        let pointee_type = self
            .pointer_intrinsic_pointee_type(call, Some(argument.operand()))
            .ok_or_else(|| {
                CodeGenerationError::InvalidMir(
                    backend_messages::call_requires_resolved_pointer_element(callee),
                )
            })?;
        let value_type = self.codegen.resolve_llvm_type(&pointee_type)?;

        self.load(value_type, pointer, "ptr_read")
    }

    pub(in crate::function) fn emit_pointer_write_intrinsic(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<(), CodeGenerationError> {
        let callee = call.name.as_str();
        let [pointer_argument, value_argument] = self.arguments(callee, arguments).exact()?;

        let pointer = pointer_argument.pointer("ptr_write_addr")?;

        let pointee_type = self.pointer_memory_element_type(call, arguments, 0)?;
        let pointee_llvm_type = self.codegen.resolve_llvm_type(&pointee_type)?;
        let value = value_argument.value(&pointee_type)?;
        let value = self.normalize_value_for_store(
            crate::function::numeric::TypedValue {
                value,
                value_type: &pointee_type,
            },
            &pointee_type,
            pointee_llvm_type,
            "ptr_write_value_adjust",
        )?;
        self.store(pointer, value)?;

        Ok(())
    }

    pub(in crate::function) fn emit_pointer_replace_intrinsic(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let callee = call.name.as_str();
        let [pointer_argument, value_argument] = self.arguments(callee, arguments).exact()?;

        let pointer = pointer_argument.pointer("ptr_replace_addr")?;
        let pointee_type = self.pointer_memory_element_type(call, arguments, 0)?;
        let pointee_llvm_type = self.codegen.resolve_llvm_type(&pointee_type)?;
        let old = self.load(pointee_llvm_type, pointer, "ptr_replace_old")?;
        let value = value_argument.value(&pointee_type)?;
        let value = self.normalize_value_for_store(
            crate::function::numeric::TypedValue {
                value,
                value_type: &pointee_type,
            },
            &pointee_type,
            pointee_llvm_type,
            "ptr_replace_value",
        )?;
        self.store(pointer, value)?;

        Ok(old)
    }

    pub(in crate::function) fn emit_pointer_swap_intrinsic(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<(), CodeGenerationError> {
        let callee = call.name.as_str();
        let [a_argument, b_argument] = self.arguments(callee, arguments).exact()?;

        let first_value = a_argument.pointer("ptr_swap_a")?;
        let second_value = b_argument.pointer("ptr_swap_b")?;
        let pointee_type = self.pointer_memory_element_type(call, arguments, 0)?;
        let pointee_llvm_type = self.codegen.resolve_llvm_type(&pointee_type)?;
        let a_value = self.load(pointee_llvm_type, first_value, "ptr_swap_a_value")?;
        let b_value = self.load(pointee_llvm_type, second_value, "ptr_swap_b_value")?;

        self.store(first_value, b_value)?;
        self.store(second_value, a_value)?;

        Ok(())
    }

    pub(in crate::function) fn emit_pointer_copy_intrinsic(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<(), CodeGenerationError> {
        let callee = call.name.as_str();
        let [source_argument, destination_argument, count_argument] =
            self.arguments(callee, arguments).exact()?;

        let source = source_argument.pointer("ptr_copy_src")?;
        let destination = destination_argument.pointer("ptr_copy_dest")?;
        let count = count_argument.integer(&Type::USize)?;
        let element_type = self.pointer_memory_element_type(call, arguments, 0)?;
        let length = self.element_count_to_bytes(count, &element_type, "ptr_copy_len")?;

        if call.kind
            == x_ast::intrinsics::BuiltinKind::Pointer(
                x_ast::intrinsics::PointerOperation::CopyNonoverlapping,
            )
        {
            self.memcpy(destination, source, length)?;
        } else {
            self.memmove(destination, source, length)?;
        }

        Ok(())
    }

    pub(in crate::function) fn emit_pointer_write_bytes_intrinsic(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<(), CodeGenerationError> {
        let callee = call.name.as_str();
        let [destination_argument, value_argument, count_argument] =
            self.arguments(callee, arguments).exact()?;

        let destination = destination_argument.pointer("ptr_write_bytes_dest")?;
        let value = value_argument.integer(&Type::U8)?;
        let count = count_argument.integer(&Type::USize)?;
        let element_type = self.pointer_memory_element_type(call, arguments, 0)?;
        let length = self.element_count_to_bytes(count, &element_type, "ptr_write_bytes_len")?;
        self.memset(destination, value, length)?;

        Ok(())
    }
}
