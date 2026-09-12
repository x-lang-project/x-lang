use inkwell::IntPredicate;
use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValue, IntValue};

use super::super::FunctionCompiler;
use super::LoopNames;
use crate::errors::CodeGenerationError;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn br(
        &self,
        destination: BasicBlock<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        self.b().build_unconditional_branch(destination)?;

        Ok(())
    }

    pub(in crate::function) fn condition_br(
        &self,
        condition: IntValue<'ctx>,
        then_block: BasicBlock<'ctx>,
        else_block: BasicBlock<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        self.b()
            .build_conditional_branch(condition, then_block, else_block)?;

        Ok(())
    }

    pub(in crate::function) fn return_value(
        &self,
        value: Option<&dyn BasicValue<'ctx>>,
    ) -> Result<(), CodeGenerationError> {
        self.b().build_return(value)?;

        Ok(())
    }

    pub(in crate::function) fn unreachable(&self) -> Result<(), CodeGenerationError> {
        self.b().build_unreachable()?;

        Ok(())
    }

    pub(in crate::function) fn trap(&self) -> Result<(), CodeGenerationError> {
        let trap = self.m().get_function("llvm.trap").unwrap_or_else(|| {
            self.m().add_function(
                "llvm.trap",
                self.context().void_type().fn_type(&[], false),
                None,
            )
        });
        self.call(trap, &[], "")?;

        self.unreachable()
    }

    pub(in crate::function) fn trap_if_false(
        &self,
        condition: IntValue<'ctx>,
        prefix: &str,
    ) -> Result<(), CodeGenerationError> {
        let function = self.current_function()?;
        let success = self
            .context()
            .append_basic_block(function, &format!("{prefix}_ok"));
        let failure = self
            .context()
            .append_basic_block(function, &format!("{prefix}_trap"));

        self.condition_br(condition, success, failure)?;

        self.b().position_at_end(failure);
        self.trap()?;

        self.b().position_at_end(success);

        Ok(())
    }

    pub(in crate::function) fn counted_loop(
        &self,
        prefix: &str,
        count: IntValue<'ctx>,
        body: &mut dyn FnMut(IntValue<'ctx>) -> Result<(), CodeGenerationError>,
    ) -> Result<(), CodeGenerationError> {
        self.counted_loop_with_names(
            count,
            LoopNames {
                prefix,
                index_slot: &format!("{prefix}_index"),
                index_load: &format!("{prefix}_i"),
                next: &format!("{prefix}_next_i"),
            },
            body,
        )
    }

    pub(in crate::function) fn counted_loop_with_names(
        &self,
        count: IntValue<'ctx>,
        names: LoopNames<'_>,
        body: &mut dyn FnMut(IntValue<'ctx>) -> Result<(), CodeGenerationError>,
    ) -> Result<(), CodeGenerationError> {
        let LoopNames {
            prefix,
            index_slot: index_slot_name,
            index_load: index_load_name,
            next: next_name,
        } = names;
        let condition_name = format!("{prefix}_cond");
        let body_name = format!("{prefix}_body");
        let end_name = format!("{prefix}_end");
        let in_bounds_name = format!("{prefix}_in_bounds");

        let index_type = count.get_type();

        let index_slot = self.alloca(index_type.into(), index_slot_name)?;
        self.store(index_slot, index_type.const_zero().into())?;

        let function = self.current_function()?;
        let condition_basic_block = self.context().append_basic_block(function, &condition_name);
        let body_basic_block = self.context().append_basic_block(function, &body_name);
        let end_basic_block = self.context().append_basic_block(function, &end_name);

        self.br(condition_basic_block)?;

        self.b().position_at_end(condition_basic_block);

        let index = self
            .load(index_type.into(), index_slot, index_load_name)?
            .into_int_value();
        let in_bounds = self.icmp(IntPredicate::ULT, index, count, &in_bounds_name)?;
        self.condition_br(in_bounds, body_basic_block, end_basic_block)?;

        self.b().position_at_end(body_basic_block);
        body(index)?;
        let next = self.integer_add(index, index_type.const_int(1, false), next_name)?;
        self.store(index_slot, next.into())?;
        self.br(condition_basic_block)?;

        self.b().position_at_end(end_basic_block);

        Ok(())
    }
}
