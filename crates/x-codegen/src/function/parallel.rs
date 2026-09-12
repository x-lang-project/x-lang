use inkwell::module;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum, StructType};
use inkwell::values::{BasicMetadataValueEnum, FunctionValue, PointerValue};

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    fn build_parallel_thunk(
        &self,
        worker_function: FunctionValue<'ctx>,
        environment_type: StructType<'ctx>,
        fields: &[BasicTypeEnum<'ctx>],
    ) -> Result<FunctionValue<'ctx>, CodeGenerationError> {
        let name = format!(
            "__x_parallel_context_{}_{}",
            self.function_name,
            self.m().get_functions().count()
        );
        let thunk = self.m().add_function(
            &name,
            self.i32()
                .fn_type(&[self.i32().into(), self.pointer_type().into()], false),
            Some(module::Linkage::Internal),
        );
        let entry = self.context().append_basic_block(thunk, "entry");
        let builder = self.context().create_builder();
        builder.position_at_end(entry);
        let input = thunk.get_nth_param(0).ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(backend_messages::missing_callback_index())
        })?;
        let context = thunk
            .get_nth_param(1)
            .ok_or_else(|| {
                CodeGenerationError::LlvmInternalError(backend_messages::missing_callback_context())
            })?
            .into_pointer_value();

        let mut arguments: Vec<BasicMetadataValueEnum<'ctx>> = vec![input.into()];

        for (index, field) in fields.iter().enumerate() {
            let slot = builder.build_struct_gep(
                environment_type,
                context,
                index as u32,
                "capture_slot",
            )?;
            arguments.push(builder.build_load(*field, slot, "capture")?.into());
        }

        let result = builder
            .build_call(worker_function, &arguments, "worker_result")?
            .try_as_basic_value()
            .basic()
            .ok_or_else(|| {
                CodeGenerationError::LlvmInternalError(
                    backend_messages::parallel_worker_returned_void(),
                )
            })?;
        builder.build_return(Some(&result))?;

        Ok(thunk)
    }
}

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
