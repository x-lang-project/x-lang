use inkwell::values::{self, BasicValueEnum, PointerValue};

use super::super::FunctionCompiler;
use super::SliceValue;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn emit_slice_descriptor(
        &self,
        data: BasicValueEnum<'ctx>,
        length: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if !data.is_pointer_value() {
            return Err(CodeGenerationError::InvalidMir(
                backend_messages::slice_descriptor_data_must_be_a_pointer(),
            ));
        }

        if !length.is_int_value() {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::slice_length_must_be_an_integer(),
            ));
        }

        let descriptor_type = self.pointer_length_type();
        let descriptor_size = descriptor_type.size_of().ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::slice_descriptor_type_has_no_concrete_size(),
            )
        })?;

        let allocate_name = x_runtime::abi::ALLOC;
        let malloc = self.required_function(allocate_name)?;
        let descriptor_size =
            self.integer_cast(descriptor_size, self.usize_type(), "slice_descriptor_size")?;
        let descriptor = self
            .b()
            .build_call(malloc, &[descriptor_size.into()], "slice_descriptor_alloc")?
            .try_as_basic_value()
            .basic()
            .ok_or_else(|| {
                CodeGenerationError::LlvmInternalError(
                    backend_messages::allocate_name_returned_void(allocate_name),
                )
            })?
            .into_pointer_value();

        let data_pointer = data.into_pointer_value();

        let length = length.into_int_value();
        let length_value = if length.get_type() != self.usize_type() {
            self.integer_cast_signed(length, self.usize_type(), false, "slice_len_zext")?
        } else {
            length
        };

        let data_slot = self.struct_gep(descriptor_type, descriptor, 0, "slice_data_slot")?;
        self.store(data_slot, data_pointer.into())?;
        let length_slot = self.struct_gep(descriptor_type, descriptor, 1, "slice_len_slot")?;
        self.store(length_slot, length_value.into())?;

        Ok(descriptor.into())
    }

    pub(in crate::function) fn load_slice_descriptor(
        &self,
        descriptor: PointerValue<'ctx>,
    ) -> Result<(PointerValue<'ctx>, values::IntValue<'ctx>), CodeGenerationError> {
        let descriptor_type = self.pointer_length_type();
        let data_slot = self.struct_gep(descriptor_type, descriptor, 0, "slice_data_slot")?;
        let length_slot = self.struct_gep(descriptor_type, descriptor, 1, "slice_len_slot")?;

        let data = self
            .load(self.pointer_type().into(), data_slot, "slice_data")?
            .into_pointer_value();
        let length = self
            .load(self.usize_type().into(), length_slot, "slice_len")?
            .into_int_value();

        Ok((data, length))
    }

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
