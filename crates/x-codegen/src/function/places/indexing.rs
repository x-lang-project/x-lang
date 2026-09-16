use inkwell::IntPredicate;
use inkwell::values::{IntValue, PointerValue};

use super::super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::{Operand, Place};

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    fn resolve_descriptor_index_pointer(
        &self,
        slot: PointerValue<'ctx>,
        index: IntValue<'ctx>,
        element: inkwell::types::BasicTypeEnum<'ctx>,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        let descriptor = self
            .load(self.pointer_type().into(), slot, "sequence_descriptor_load")?
            .into_pointer_value();
        let descriptor_type = self.pointer_length_type();
        let data_slot = self.struct_gep(descriptor_type, descriptor, 0, "sequence_data_slot")?;
        let data = self
            .load(self.pointer_type().into(), data_slot, "sequence_index_data")?
            .into_pointer_value();
        let length_slot = self.struct_gep(descriptor_type, descriptor, 1, "sequence_len_slot")?;
        let length = self
            .load(self.usize_type().into(), length_slot, "sequence_index_len")?
            .into_int_value();
        let index =
            self.integer_cast_signed(index, self.usize_type(), false, "sequence_index_usize")?;
        self.check_index_bounds(index, length, "sequence_index_in_bounds", "sequence_bounds")?;

        self.gep(element, data, index, "sequence_index_gep")
    }

    pub(super) fn resolve_fixed_array_index_pointer(
        &self,
        base_type: &Type,
        base_pointer: PointerValue<'ctx>,
        index_value: IntValue<'ctx>,
        length: &usize,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        unsafe {
            if let Type::FixedSizedArray(element_type, _) = base_type
                && self.codegen.soa_fields(element_type).is_some()
            {
                return Err(CodeGenerationError::LlvmInternalError(backend_messages::a_layout_soa_array_element_has_no_contiguous_whole_value_address_project()));
            }

            let array_type = self.codegen.resolve_llvm_type(base_type)?;
            let zero = index_value.get_type().const_zero();
            let length = index_value.get_type().const_int(*length as u64, false);
            self.check_index_bounds(
                index_value,
                length,
                "fixed_array_index_in_bounds",
                "fixed_array_bounds",
            )?;

            self.b()
                .build_gep(
                    array_type,
                    base_pointer,
                    &[zero, index_value],
                    "fixed_array_elem_gep",
                )
                .map_err(Into::into)
        }
    }

    pub(super) fn check_index_bounds(
        &self,
        index: IntValue<'ctx>,
        length: IntValue<'ctx>,
        comparison_name: &str,
        trap_name: &str,
    ) -> Result<(), CodeGenerationError> {
        let in_bounds = self.icmp(IntPredicate::ULT, index, length, comparison_name)?;

        self.trap_if_false(in_bounds, trap_name)
    }

    pub(super) fn resolve_index_pointer(
        &self,
        base: &Place,
        index: &Operand,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        let mut base_pointer = self.resolve_place_pointer(base)?;
        let index_value = self
            .load_operand_value_with_type(index, &Type::USize)?
            .into_int_value();

        let mut base_type = self.get_place_type(base);

        while let Type::Reference { inner, .. } = base_type {
            base_pointer = self
                .load(
                    self.pointer_type().into(),
                    base_pointer,
                    "indexed_reference_load",
                )?
                .into_pointer_value();
            base_type = *inner;
        }

        unsafe {
            match &base_type {
                Type::FixedSizedArray(_, length) => self.resolve_fixed_array_index_pointer(
                    &base_type,
                    base_pointer,
                    index_value,
                    length,
                ),

                Type::Array(element) => self.resolve_descriptor_index_pointer(
                    base_pointer,
                    index_value,
                    self.codegen.resolve_llvm_type(element)?,
                ),

                Type::RawPointer { inner, .. } => {
                    let pointer_type = self.pointer_type();
                    let data_pointer = self
                        .load(pointer_type.into(), base_pointer, "indexed_base_load")?
                        .into_pointer_value();

                    let element_type = self.codegen.resolve_llvm_type(inner)?;

                    self.b()
                        .build_gep(
                            element_type,
                            data_pointer,
                            &[index_value],
                            "ptr_array_elem_gep",
                        )
                        .map_err(Into::into)
                }

                Type::String => Err(CodeGenerationError::UnsupportedType(base_type.clone())),

                _ => Err(CodeGenerationError::UnsupportedType(base_type.clone())),
            }
        }
    }
}
