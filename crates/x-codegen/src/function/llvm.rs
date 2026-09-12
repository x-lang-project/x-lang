mod arithmetic;
mod control_flow;
mod memory;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types;
use inkwell::types::{BasicTypeEnum, IntType, PointerType, StructType};
use inkwell::values::{
    AggregateValueEnum, BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, FunctionValue,
    GlobalValue, IntValue, PointerValue, StructValue,
};

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;

pub(super) struct LoopNames<'a> {
    pub prefix: &'a str,
    pub index_slot: &'a str,
    pub index_load: &'a str,
    pub next: &'a str,
}

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn b(&self) -> &Builder<'ctx> {
        &self.codegen.builder
    }

    pub(super) fn context(&self) -> &'ctx Context {
        self.codegen.context
    }

    pub(super) fn m(&self) -> &Module<'ctx> {
        &self.codegen.module
    }

    pub(super) fn i8(&self) -> types::IntType<'ctx> {
        self.context().i8_type()
    }

    pub(super) fn i32(&self) -> types::IntType<'ctx> {
        self.context().i32_type()
    }

    pub(super) fn i64(&self) -> types::IntType<'ctx> {
        self.context().i64_type()
    }

    pub(super) fn usize_type(&self) -> types::IntType<'ctx> {
        self.context()
            .ptr_sized_int_type(&self.codegen.target_data(), None)
    }

    pub(super) fn boolean_type(&self) -> types::IntType<'ctx> {
        self.context().bool_type()
    }

    pub(super) fn pointer_type(&self) -> types::PointerType<'ctx> {
        self.context()
            .ptr_type(self.codegen.pointer_address_space())
    }

    pub(super) fn pointer_length_type(&self) -> StructType<'ctx> {
        self.context().struct_type(
            &[self.pointer_type().into(), self.usize_type().into()],
            false,
        )
    }

    pub(super) fn zero_i8(&self) -> BasicValueEnum<'ctx> {
        self.i8().const_zero().into()
    }

    pub(super) fn store(
        &self,
        pointer: PointerValue<'ctx>,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        self.b().build_store(pointer, value)?;

        Ok(())
    }

    pub(super) fn load(
        &self,
        value_type: BasicTypeEnum<'ctx>,
        pointer: PointerValue<'ctx>,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        self.b()
            .build_load(value_type, pointer, name)
            .map_err(Into::into)
    }

    pub(super) fn alloca(
        &self,
        value_type: BasicTypeEnum<'ctx>,
        name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        let current_block = self.b().get_insert_block();
        let entry_block = current_block
            .and_then(|block| block.get_parent())
            .and_then(|function| function.get_first_basic_block());

        if let Some(entry_block) = entry_block {
            if let Some(terminator) = entry_block.get_terminator() {
                self.b().position_before(&terminator);
            } else {
                self.b().position_at_end(entry_block);
            }
        }

        let alloca = self
            .b()
            .build_alloca(value_type, name)
            .map_err(CodeGenerationError::from);

        if let Some(block) = current_block {
            self.b().position_at_end(block);
        }

        alloca
    }

    pub(super) fn pointer_to_integer(
        &self,
        pointer: PointerValue<'ctx>,
        value_type: IntType<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_ptr_to_int(pointer, value_type, name)
            .map_err(Into::into)
    }

    pub(super) fn integer_to_pointer(
        &self,
        value: IntValue<'ctx>,
        value_type: PointerType<'ctx>,
        name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_int_to_ptr(value, value_type, name)
            .map_err(Into::into)
    }

    pub(super) fn struct_gep(
        &self,
        value_type: StructType<'ctx>,
        pointer: PointerValue<'ctx>,
        index: u32,
        name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_struct_gep(value_type, pointer, index, name)
            .map_err(Into::into)
    }

    pub(super) fn insert_value(
        &self,
        aggregate: AggregateValueEnum<'ctx>,
        value: BasicValueEnum<'ctx>,
        index: u32,
        name: &str,
    ) -> Result<AggregateValueEnum<'ctx>, CodeGenerationError> {
        self.b()
            .build_insert_value(aggregate, value, index, name)
            .map_err(Into::into)
    }

    pub(super) fn struct_value(
        &self,
        value_type: StructType<'ctx>,
        fields: &[(BasicValueEnum<'ctx>, &str)],
    ) -> Result<StructValue<'ctx>, CodeGenerationError> {
        let mut value = value_type.get_undef();

        for (index, (field, name)) in fields.iter().enumerate() {
            value = self
                .insert_value(value.into(), *field, index as u32, name)?
                .into_struct_value();
        }

        Ok(value)
    }

    pub(super) fn call(
        &self,
        callee: FunctionValue<'ctx>,
        arguments: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) -> Result<CallSiteValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_call(callee, arguments, name)
            .map_err(Into::into)
    }

    pub(super) fn global_string_pointer(
        &self,
        value: &str,
        name: &str,
    ) -> Result<GlobalValue<'ctx>, CodeGenerationError> {
        self.b()
            .build_global_string_ptr(value, name)
            .map_err(Into::into)
    }
}
