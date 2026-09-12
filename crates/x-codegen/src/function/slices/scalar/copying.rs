use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function::slices) fn emit_typed_slice_fill(
        &self,
        data: PointerValue<'ctx>,
        length: IntValue<'ctx>,
        element: BasicTypeEnum<'ctx>,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        self.counted_loop("slice_fill", length, &mut |index| {
            let pointer = self.gep(element, data, index, "slice_fill_elem")?;

            self.store(pointer, value)
        })
    }

    pub(in crate::function::slices) fn emit_typed_copy_loop(
        &self,
        element: BasicTypeEnum<'ctx>,
        source: PointerValue<'ctx>,
        destination: PointerValue<'ctx>,
        count: IntValue<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        self.counted_loop("slice_copy", count, &mut |index| {
            let source = self.gep(element, source, index, "slice_copy_src_elem")?;
            let destination = self.gep(element, destination, index, "slice_copy_dst_elem")?;
            let value = self.load(element, source, "slice_copy_value")?;

            self.store(destination, value)
        })
    }

    pub(in crate::function::slices) fn emit_typed_copy_within_loop(
        &self,
        element: BasicTypeEnum<'ctx>,
        source: CopyEndpoint<'ctx>,
        destination: CopyEndpoint<'ctx>,
        count: IntValue<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        let CopyEndpoint {
            pointer: current_source,
            index: source_index,
        } = source;
        let CopyEndpoint {
            pointer: current_destination,
            index: destination_index,
        } = destination;

        let function = self.current_function()?;
        let backward_basic_block = self
            .context()
            .append_basic_block(function, "slice_copy_backward");
        let forward_basic_block = self
            .context()
            .append_basic_block(function, "slice_copy_forward");
        let end_basic_block = self
            .context()
            .append_basic_block(function, "slice_copy_within_end");

        let copy_backward = self.icmp(
            IntPredicate::UGT,
            destination_index,
            source_index,
            "slice_copy_needs_backward",
        )?;
        self.b().build_conditional_branch(
            copy_backward,
            backward_basic_block,
            forward_basic_block,
        )?;

        self.b().position_at_end(forward_basic_block);
        self.emit_typed_copy_loop(element, current_source, current_destination, count)?;
        self.br(end_basic_block)?;

        self.b().position_at_end(backward_basic_block);
        let index_type = count.get_type();

        let index_slot = self.alloca(index_type.into(), "slice_copy_back_index")?;
        self.store(index_slot, count.into())?;

        let condition_basic_block = self
            .context()
            .append_basic_block(function, "slice_copy_back_cond");
        let body_basic_block = self
            .context()
            .append_basic_block(function, "slice_copy_back_body");

        self.br(condition_basic_block)?;

        self.b().position_at_end(condition_basic_block);
        let index = self
            .load(index_type.into(), index_slot, "slice_copy_back_i")?
            .into_int_value();
        let has_items = self.icmp(
            IntPredicate::UGT,
            index,
            index_type.const_zero(),
            "slice_copy_back_has_items",
        )?;
        self.b()
            .build_conditional_branch(has_items, body_basic_block, end_basic_block)?;

        self.b().position_at_end(body_basic_block);
        let element_index = self.integer_sub(
            index,
            index_type.const_int(1, false),
            "slice_copy_back_elem_i",
        )?;
        let source_element = self.gep(
            element,
            current_source,
            element_index,
            "slice_copy_back_src_elem",
        )?;
        let destination_element = self.gep(
            element,
            current_destination,
            element_index,
            "slice_copy_back_dst_elem",
        )?;

        let value = self.load(element, source_element, "slice_copy_back_value")?;
        self.store(destination_element, value)?;

        self.store(index_slot, element_index.into())?;
        self.br(condition_basic_block)?;

        self.b().position_at_end(end_basic_block);

        Ok(())
    }
}
