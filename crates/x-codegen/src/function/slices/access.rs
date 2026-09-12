use inkwell::IntPredicate;
use inkwell::values::BasicValueEnum;

use super::super::FunctionCompiler;
use super::SliceValue;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_slice_is_empty(
        &self,
        slice: SliceValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        self.icmp(
            IntPredicate::EQ,
            slice.len,
            slice.len.get_type().const_zero(),
            "slice_is_empty",
        )
        .map(|value| value.into())
    }

    pub(super) fn emit_slice_last(
        &self,
        slice: SliceValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let one = slice.len.get_type().const_int(1, false);
        let has_items = self.icmp(
            IntPredicate::NE,
            slice.len,
            slice.len.get_type().const_zero(),
            "slice_last_has_items",
        )?;

        let index = self.integer_sub(slice.len, one, "slice_last_index")?;
        let pointer = self.gep(slice.element, slice.data, index, "slice_last_ptr")?;

        self.option_pointer(has_items, pointer, "slice_last_option")
    }
}
