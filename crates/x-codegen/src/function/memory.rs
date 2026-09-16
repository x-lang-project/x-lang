mod copying;
use inkwell::values;
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use super::llvm;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_ast::intrinsics::{BuiltinCall, BuiltinKind, CoreMemoryOperation};
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_mem_swap(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<(), CodeGenerationError> {
        let function = call.name.as_str();
        let [a_argument, b_argument] = self.arguments(function, arguments).exact()?;
        let value_type = self.pointer_memory_element_type(call, arguments, 0)?;
        let llvm_type = self.codegen.resolve_llvm_type(&value_type)?;
        let first_value = a_argument.pointer("mem_swap_a_ptr")?;
        let second_value = b_argument.pointer("mem_swap_b_ptr")?;
        let a_value = self.load(llvm_type, first_value, "mem_swap_a")?;
        let b_value = self.load(llvm_type, second_value, "mem_swap_b")?;

        self.store(first_value, b_value)?;
        self.store(second_value, a_value)?;

        Ok(())
    }

    pub(super) fn emit_mem_replace(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let callee = call.name.as_str();
        let [place_argument, value_argument] = self.arguments(callee, arguments).exact()?;
        let value_type = self.pointer_memory_element_type(call, arguments, 0)?;
        let llvm_type = self.codegen.resolve_llvm_type(&value_type)?;
        let place = place_argument.pointer("mem_replace_ptr")?;
        let new_value = value_argument.value(&value_type)?;
        let new_value = self.normalize_value_for_store(
            crate::function::numeric::TypedValue {
                value: new_value,
                value_type: &value_type,
            },
            &value_type,
            llvm_type,
            "mem_replace_value",
        )?;
        let old = self.load(llvm_type, place, "mem_replace_old")?;
        self.store(place, new_value)?;

        Ok(old)
    }

    pub(super) fn emit_memory_intrinsic(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<bool, CodeGenerationError> {
        let function = call.name.as_str();

        match call.kind {
            BuiltinKind::Intrinsic(x_ast::intrinsics::Intrinsic::Memory(
                operation @ (x_ast::MemoryIntrinsic::Copy | x_ast::MemoryIntrinsic::Move),
            )) => {
                let [destination_argument, source_argument, length_argument] =
                    Self::expect_arguments(function, arguments)?;

                let destination = self
                    .load_operand_value(destination_argument)?
                    .into_pointer_value();
                let source = self
                    .load_operand_value(source_argument)?
                    .into_pointer_value();
                let length = self.integer_argument(length_argument, &Type::USize)?;

                if operation == x_ast::MemoryIntrinsic::Copy {
                    self.memcpy(destination, source, length)?;
                } else {
                    self.memmove(destination, source, length)?;
                }

                Ok(true)
            }
            BuiltinKind::Intrinsic(x_ast::intrinsics::Intrinsic::Memory(
                x_ast::MemoryIntrinsic::Set,
            )) => {
                let [destination_argument, value_argument, length_argument] =
                    Self::expect_arguments(function, arguments)?;

                let destination = self
                    .load_operand_value(destination_argument)?
                    .into_pointer_value();
                let value = self.integer_argument(value_argument, &Type::U8)?;
                let length = self.integer_argument(length_argument, &Type::USize)?;

                self.memset(destination, value, length)?;

                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(super) fn emit_core_mem_method(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        let function = call.name.as_str();
        let BuiltinKind::CoreMemory(operation) = call.kind else {
            return Err(CodeGenerationError::MissingCalleeSymbol(
                function.to_string(),
            ));
        };

        match operation {
            CoreMemoryOperation::CopyBytesNonoverlapping
            | CoreMemoryOperation::CopyBytes
            | CoreMemoryOperation::MoveBytes => {
                self.emit_core_mem_byte_copy(call, arguments, operation)
            }
            CoreMemoryOperation::SetBytes => {
                let [destination_argument, value_argument, count_argument] =
                    Self::expect_arguments(function, arguments)?;
                let destination = self.pointer_argument(destination_argument, "mem_set_dest")?;
                let value = self.integer_argument(value_argument, &Type::U8)?;
                let length = self.integer_argument(count_argument, &Type::USize)?;
                self.memset(destination, value, length)?;

                Ok(None)
            }
            CoreMemoryOperation::ZeroBytes => {
                let [destination_argument, count_argument] =
                    Self::expect_arguments(function, arguments)?;
                let destination = self.pointer_argument(destination_argument, "mem_zero_dest")?;
                let length = self.integer_argument(count_argument, &Type::USize)?;
                self.memset(destination, self.i8().const_zero(), length)?;

                Ok(None)
            }
            CoreMemoryOperation::CopyObjectsNonoverlapping | CoreMemoryOperation::MoveObjects => {
                self.emit_core_mem_object_copy(call, arguments, operation)
            }
            CoreMemoryOperation::ZeroObjects => {
                let [destination_argument, count_argument] =
                    Self::expect_arguments(function, arguments)?;
                let destination = self.pointer_argument(destination_argument, "mem_zero_dest")?;
                let count = self.integer_argument(count_argument, &Type::USize)?;
                let element_type = self.core_mem_element_type(call, arguments)?;
                let length = self.element_count_to_bytes(count, &element_type, "mem_zero_len")?;
                self.memset(destination, self.i8().const_zero(), length)?;

                Ok(None)
            }
            CoreMemoryOperation::Fill => self.emit_core_mem_fill(call, arguments),
            CoreMemoryOperation::CheckedArraySize => {
                let [count_argument] = Self::expect_arguments(function, arguments)?;
                let count = self.integer_argument(count_argument, &Type::USize)?;
                let element_type = self.core_mem_element_type(call, arguments)?;
                let length = self.element_count_to_bytes(count, &element_type, "mem_array_size")?;

                Ok(Some(length.into()))
            }
        }
    }

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
