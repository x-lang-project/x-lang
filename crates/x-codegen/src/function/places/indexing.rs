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
}
