use inkwell::values::FunctionValue;

use crate::codegen::CodeGeneration;
use crate::errors::CodeGenerationError;
use crate::function::LocalFrame;
use x_mir::{LocalIdentifier, MirBody};

impl<'ctx> CodeGeneration<'ctx> {
    pub(in crate::codegen) fn allocate_frame_slots(
        &self,
        body: &MirBody,
    ) -> Result<LocalFrame<'ctx>, CodeGenerationError> {
        let mut frame = LocalFrame::default();
        let mut locals: Vec<_> = body.locals.iter().collect();
        locals.sort_by_key(|(identifier, _)| identifier.0);
        for (identifier, declaration) in locals {
            let llvm_type = self.resolve_llvm_type(&declaration.value_type)?;
            let name = LocalFrame::storage_name(body, *identifier);
            let alloca = self.builder.build_alloca(llvm_type, &name)?;
            frame.insert(*identifier, declaration.value_type.clone(), alloca);
        }

        Ok(frame)
    }

    pub(in crate::codegen) fn bind_function_parameters(
        &self,
        function: FunctionValue<'ctx>,
        body: &MirBody,
        frame: &LocalFrame<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        for index in 0..body.argument_count {
            let parameter_value = function
                .get_nth_param(index as u32)
                .ok_or(CodeGenerationError::MissingHardwareArgument(index as u32))?;
            let target_pointer = frame
                .pointer(LocalIdentifier(index + 1))
                .ok_or(CodeGenerationError::MissingLocalAllocation(index + 1))?;
            self.builder.build_store(target_pointer, parameter_value)?;
        }

        Ok(())
    }
}
