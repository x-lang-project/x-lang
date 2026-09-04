use inkwell::module;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum, StructType};
use inkwell::values::{BasicMetadataValueEnum, FunctionValue, PointerValue};

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::Operand;

fn parallel_capture_types<'ctx>(
    parameters: &[BasicMetadataTypeEnum<'ctx>],
) -> Result<Vec<BasicTypeEnum<'ctx>>, CodeGenerationError> {
    parameters
        .iter()
        .skip(1)
        .map(|parameter| {
            BasicTypeEnum::try_from(*parameter).map_err(|_| {
                CodeGenerationError::LlvmInternalError(
                    backend_messages::unsupported_parallel_capture_type(),
                )
            })
        })
        .collect()
}
