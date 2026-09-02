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
}
