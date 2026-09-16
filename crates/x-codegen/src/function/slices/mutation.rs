use inkwell::IntPredicate;
use inkwell::values::BasicValueEnum;

use super::super::FunctionCompiler;
use super::{CopyEndpoint, SliceValue};
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_slice_fill(
        &self,
        function: &str,
        arguments: &[Operand],
        slice: SliceValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let [_, value] = self.arguments(function, arguments).exact()?;
        let value = self.load_operand_value_with_type(value.operand(), &slice.element_ast)?;
        self.emit_typed_slice_fill(slice.data, slice.len, slice.element, value)?;

        Ok(self.zero_i8())
    }

    pub(super) fn emit_slice_swap(
        &self,
        function: &str,
        arguments: &[Operand],
        slice: SliceValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let [_, first_value, second_value] = self.arguments(function, arguments).exact()?;
        let first_value = first_value.integer(&Type::USize)?;
        let second_value = second_value.integer(&Type::USize)?;

        for (index, name) in [
            (first_value, "slice_swap_a_bounds"),
            (second_value, "slice_swap_b_bounds"),
        ] {
            let valid = self.icmp(IntPredicate::ULT, index, slice.len, "slice_swap_in_bounds")?;
            self.trap_if_false(valid, name)?;
        }

        let element = slice.element;
        let a_pointer = self.gep(element, slice.data, first_value, "slice_swap_a")?;
        let b_pointer = self.gep(element, slice.data, second_value, "slice_swap_b")?;

        let a_value = self.load(element, a_pointer, "slice_swap_a_val")?;
        let b_value = self.load(element, b_pointer, "slice_swap_b_val")?;

        self.store(a_pointer, b_value)?;
        self.store(b_pointer, a_value)?;

        Ok(self.zero_i8())
    }

    pub(super) fn emit_slice_copy_within(
        &self,
        function: &str,
        arguments: &[Operand],
        slice: SliceValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let [_, start, end, destination] = self.arguments(function, arguments).exact()?;
        let start = start.integer(&Type::USize)?;
        let end = end.integer(&Type::USize)?;
        let destination = destination.integer(&Type::USize)?;

        let ordered = self.icmp(IntPredicate::ULE, start, end, "slice_copy_range_ordered")?;
        self.trap_if_false(ordered, "slice_copy_range_order")?;
        let source_fits = self.icmp(IntPredicate::ULE, end, slice.len, "slice_copy_source_fits")?;
        self.trap_if_false(source_fits, "slice_copy_source_bounds")?;
        let destination_valid = self.icmp(
            IntPredicate::ULE,
            destination,
            slice.len,
            "slice_copy_destination_valid",
        )?;
        self.trap_if_false(destination_valid, "slice_copy_destination_bounds")?;
        let count = self.integer_sub(end, start, "slice_copy_within_count")?;
        let destination_space =
            self.integer_sub(slice.len, destination, "slice_copy_destination_space")?;
        let destination_fits = self.icmp(
            IntPredicate::ULE,
            count,
            destination_space,
            "slice_copy_destination_fits",
        )?;
        self.trap_if_false(destination_fits, "slice_copy_destination_range")?;
        let source_pointer = self.gep(slice.element, slice.data, start, "slice_copy_within_src")?;
        let destination_pointer = self.gep(
            slice.element,
            slice.data,
            destination,
            "slice_copy_within_dst",
        )?;

        self.emit_typed_copy_within_loop(
            slice.element,
            CopyEndpoint {
                pointer: source_pointer,
                index: start,
            },
            CopyEndpoint {
                pointer: destination_pointer,
                index: destination,
            },
            count,
        )?;

        Ok(self.zero_i8())
    }

    pub(super) fn emit_slice_copy_from(
        &self,
        function: &str,
        arguments: &[Operand],
        slice: SliceValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let [_, source] = self.arguments(function, arguments).exact()?;
        let source = self.slice_receiver(function, &[source.operand().clone()])?;

        if source.element != slice.element {
            return Err(CodeGenerationError::UnsupportedType(source.element_ast));
        }

        let (source_data, source_length) = (source.data, source.len);

        let copy_all = self.icmp(
            IntPredicate::ULE,
            source_length,
            slice.len,
            "slice_copy_from_src_fits",
        )?;
        let count =
            self.select_integer(copy_all, source_length, slice.len, "slice_copy_from_len")?;

        self.emit_typed_copy_loop(slice.element, source_data, slice.data, count)?;

        Ok(self.zero_i8())
    }
}
