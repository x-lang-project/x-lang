use inkwell::module;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum, StructType};
use inkwell::values::{BasicMetadataValueEnum, FunctionValue, PointerValue};

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_parallel_for(
        &self,
        start: &Operand,
        end: &Operand,
        worker: &str,
        captures: &[Operand],
    ) -> Result<(), CodeGenerationError> {
        let worker_function = self.resolve_callee(worker, Some(1 + captures.len()))?;
        let worker_type = worker_function.get_type();
        let parameters = worker_type.get_param_types();

        if parameters.first() != Some(&self.i32().into())
            || parameters.len() != 1 + captures.len()
            || worker_type.get_return_type() != Some(self.i32().into())
        {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::parallel_worker_worker_must_have_the_exact_signature_i_i(worker),
            ));
        }

        let start = self
            .load_operand_value_with_type(start, &Type::I32)?
            .into_int_value();
        let end = self
            .load_operand_value_with_type(end, &Type::I32)?
            .into_int_value();

        if !captures.is_empty() {
            let fields = parallel_capture_types(&parameters)?;
            let environment_type = self.context().struct_type(&fields, false);
            let environment = self.build_parallel_environment(
                worker_function,
                worker,
                captures,
                environment_type,
            )?;
            let thunk = self.build_parallel_thunk(worker_function, environment_type, &fields)?;
            let dispatch = self.codegen.parallel_dispatch_function(true);

            self.call(
                dispatch,
                &[
                    start.into(),
                    end.into(),
                    thunk.as_global_value().as_pointer_value().into(),
                    environment.into(),
                ],
                "parallel_context_dispatch",
            )?;

            return Ok(());
        }

        let dispatch = self.codegen.parallel_dispatch_function(false);
        let arguments: [BasicMetadataValueEnum<'ctx>; 3] = [
            start.into(),
            end.into(),
            worker_function.as_global_value().as_pointer_value().into(),
        ];

        self.call(dispatch, &arguments, "parallel_dispatch")?;

        Ok(())
    }

    fn build_parallel_environment(
        &self,
        worker_function: FunctionValue<'ctx>,
        worker: &str,
        captures: &[Operand],
        environment_type: StructType<'ctx>,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        let environment = self.alloca(environment_type.into(), "parallel_environment")?;

        for (index, capture) in captures.iter().enumerate() {
            let value = self.load_operand_value(capture)?;
            let parameter = worker_function
                .get_nth_param((index + 1) as u32)
                .ok_or_else(|| {
                    CodeGenerationError::LlvmInternalError(
                        backend_messages::missing_parallel_worker_parameter(),
                    )
                })?;
            let value = self.normalize_call_argument(
                crate::function::numeric::TypedValue {
                    value,
                    value_type: &self.body.operand_type(capture),
                },
                parameter,
                worker,
                index + 1,
            )?;
            let slot = self.struct_gep(
                environment_type,
                environment,
                index as u32,
                "parallel_capture_slot",
            )?;
            self.store(slot, value)?;
        }

        Ok(environment)
    }

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
