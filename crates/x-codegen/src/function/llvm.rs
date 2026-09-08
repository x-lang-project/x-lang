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
}
