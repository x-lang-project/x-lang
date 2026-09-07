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

    pub(super) fn i64(&self) -> types::IntType<'ctx> {
        self.context().i64_type()
    }

    pub(super) fn boolean_type(&self) -> types::IntType<'ctx> {
        self.context().bool_type()
    }
}
