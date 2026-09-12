use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn emit_i32_slice_contains(
        &self,
        data: PointerValue<'ctx>,
        length: values::IntValue<'ctx>,
        needle: values::IntValue<'ctx>,
    ) -> Result<values::IntValue<'ctx>, CodeGenerationError> {
        let i32_type = self.i32();
        let boolean_type = self.boolean_type();
        let index_type = length.get_type();

        let result_slot = self.alloca(boolean_type.into(), "slice_pred_result")?;
        let index_slot = self.alloca(index_type.into(), "slice_pred_index")?;

        self.store(result_slot, boolean_type.const_zero().into())?;
        self.store(index_slot, index_type.const_zero().into())?;

        let function = self.current_function()?;
        let chunk_condition = self
            .context()
            .append_basic_block(function, "slice_pred_chunk_cond");
        let chunk_body = self
            .context()
            .append_basic_block(function, "slice_pred_chunk_body");
        let tail_condition = self
            .context()
            .append_basic_block(function, "slice_pred_tail_cond");
        let tail_body = self
            .context()
            .append_basic_block(function, "slice_pred_tail_body");
        let end = self
            .context()
            .append_basic_block(function, "slice_pred_end");

        self.br(chunk_condition)?;

        self.b().position_at_end(chunk_condition);
        let index = self
            .load(index_type.into(), index_slot, "slice_pred_i")?
            .into_int_value();
        let remaining = self.integer_sub(length, index, "slice_pred_remaining")?;
        let full_chunk = self.icmp(
            IntPredicate::UGE,
            remaining,
            index_type.const_int(16, false),
            "slice_pred_full_chunk",
        )?;
        self.condition_br(full_chunk, chunk_body, tail_condition)?;

        self.b().position_at_end(chunk_body);
        let mut found = boolean_type.const_zero();

        for offset in 0..16 {
            let item_index = self.integer_add(
                index,
                index_type.const_int(offset, false),
                "slice_pred_chunk_i",
            )?;
            let pointer = self.gep(i32_type.into(), data, item_index, "slice_pred_elem_ptr")?;
            let value = self
                .load(i32_type.into(), pointer, "slice_pred_elem")?
                .into_int_value();

            let equal = self.icmp(IntPredicate::EQ, value, needle, "slice_contains_eq")?;
            found = self.integer_or(found, equal, "slice_contains_acc")?;
        }

        self.store(result_slot, found.into())?;

        let next = self.integer_add(
            index,
            index_type.const_int(16, false),
            "slice_pred_next_chunk",
        )?;
        self.store(index_slot, next.into())?;
        self.condition_br(found, end, chunk_condition)?;

        self.b().position_at_end(tail_condition);
        let index = self
            .load(index_type.into(), index_slot, "slice_pred_tail_i")?
            .into_int_value();
        let in_bounds = self.icmp(IntPredicate::ULT, index, length, "slice_pred_in_bounds")?;
        self.condition_br(in_bounds, tail_body, end)?;

        self.b().position_at_end(tail_body);
        let pointer = self.gep(i32_type.into(), data, index, "slice_pred_tail_ptr")?;
        let value = self
            .load(i32_type.into(), pointer, "slice_pred_tail_value")?
            .into_int_value();

        let equal = self.icmp(IntPredicate::EQ, value, needle, "slice_pred_tail_equal")?;

        let current = self
            .load(boolean_type.into(), result_slot, "slice_pred_tail_current")?
            .into_int_value();
        let found = self.integer_or(current, equal, "slice_pred_tail_found")?;
        self.store(result_slot, found.into())?;

        let next = self.integer_add(index, index_type.const_int(1, false), "slice_pred_next")?;
        self.store(index_slot, next.into())?;
        self.br(tail_condition)?;

        self.b().position_at_end(end);

        self.load(boolean_type.into(), result_slot, "slice_pred_result")
            .map(|element| element.into_int_value())
    }

    pub(in crate::function) fn emit_i32_slice_has_duplicate(
        &self,
        data: PointerValue<'ctx>,
        length: values::IntValue<'ctx>,
    ) -> Result<values::IntValue<'ctx>, CodeGenerationError> {
        let index_type = length.get_type();
        let i32_type = self.i32();
        let boolean_type = self.boolean_type();

        let outer_slot = self.alloca(index_type.into(), "slice_dup_outer")?;
        let result_slot = self.alloca(boolean_type.into(), "slice_dup_result")?;

        self.store(outer_slot, index_type.const_zero().into())?;
        self.store(result_slot, boolean_type.const_zero().into())?;

        let function = self.current_function()?;
        let outer_condition_basic_block = self
            .context()
            .append_basic_block(function, "slice_dup_outer_cond");
        let inner_initialise_basic_block = self
            .context()
            .append_basic_block(function, "slice_dup_inner_init");
        let outer_next_basic_block = self
            .context()
            .append_basic_block(function, "slice_dup_outer_next");
        let suffix_search_basic_block = self
            .context()
            .append_basic_block(function, "slice_dup_suffix_search");
        let end_basic_block = self.context().append_basic_block(function, "slice_dup_end");

        self.br(outer_condition_basic_block)?;

        self.b().position_at_end(outer_condition_basic_block);
        let outer = self
            .load(index_type.into(), outer_slot, "slice_dup_i")?
            .into_int_value();
        let first_inner =
            self.integer_add(outer, index_type.const_int(1, false), "slice_dup_j_start")?;
        let suffix_exists = self.icmp(
            IntPredicate::ULT,
            first_inner,
            length,
            "slice_dup_suffix_exists",
        )?;
        self.condition_br(suffix_exists, inner_initialise_basic_block, end_basic_block)?;

        self.b().position_at_end(inner_initialise_basic_block);
        let left_pointer = self.gep(i32_type.into(), data, outer, "slice_dup_left_ptr")?;
        let left = self
            .load(i32_type.into(), left_pointer, "slice_dup_left")?
            .into_int_value();
        let suffix = self.gep(i32_type.into(), data, first_inner, "slice_dup_suffix")?;
        let adjacent = self
            .load(i32_type.into(), suffix, "slice_dup_adjacent")?
            .into_int_value();
        let adjacent_matches =
            self.icmp(IntPredicate::EQ, left, adjacent, "slice_dup_adjacent_match")?;

        self.store(result_slot, adjacent_matches.into())?;
        self.condition_br(adjacent_matches, end_basic_block, suffix_search_basic_block)?;

        self.b().position_at_end(suffix_search_basic_block);
        let search_start = self.integer_add(
            first_inner,
            index_type.const_int(1, false),
            "slice_dup_search_start",
        )?;
        let suffix = self.gep(
            i32_type.into(),
            data,
            search_start,
            "slice_dup_search_suffix",
        )?;
        let suffix_length = self.integer_sub(length, search_start, "slice_dup_suffix_len")?;

        let found = self.emit_i32_slice_contains(suffix, suffix_length, left)?;

        self.store(result_slot, found.into())?;
        self.condition_br(found, end_basic_block, outer_next_basic_block)?;

        self.b().position_at_end(outer_next_basic_block);
        self.store(outer_slot, first_inner.into())?;
        self.br(outer_condition_basic_block)?;

        self.b().position_at_end(end_basic_block);

        self.load(boolean_type.into(), result_slot, "slice_dup_result")
            .map(|value| value.into_int_value())
    }
}
