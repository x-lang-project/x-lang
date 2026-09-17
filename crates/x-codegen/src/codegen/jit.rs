use inkwell::execution_engine::{JitFunction, UnsafeFunctionPointer};
use inkwell::targets;

use super::CodeGeneration;
use crate::errors::CodeGenerationError;
use x_diagnostics::backend_messages;

impl<'ctx> CodeGeneration<'ctx> {
    // The caller must ensure F exactly matches the symbol's ABI.
    unsafe fn jit_function<F: UnsafeFunctionPointer>(
        &self,
        name: &str,
    ) -> Result<JitFunction<'ctx, F>, CodeGenerationError> {
        let mut engine = self.execution_engine.borrow_mut();

        if engine.is_none() {
            let layout = self
                .module
                .get_data_layout()
                .as_str()
                .to_string_lossy()
                .into_owned();

            let created = self
                .module
                .create_jit_execution_engine(self.jit_optimization_level)
                .map_err(|error| {
                    CodeGenerationError::JitInitializationFailure(error.to_string())
                })?;
            // Engine creation changes the module layout; refresh Inkwell's view.
            self.module
                .set_data_layout(&targets::TargetData::create(&layout).get_data_layout());
            *engine = Some(created);
        }

        let active = engine.as_ref().ok_or_else(|| {
            CodeGenerationError::JitInitializationFailure(
                backend_messages::missing_execution_engine(),
            )
        })?;

        unsafe { active.get_function(name) }
            .map_err(|_| CodeGenerationError::JitExecutionFailure(name.to_string()))
    }

    /// # Safety
    /// The symbol must have signature `unsafe extern "C" fn()`.
    pub unsafe fn call_jit_function_void(&self, name: &str) -> Result<(), CodeGenerationError> {
        let function = unsafe { self.jit_function::<unsafe extern "C" fn()>(name) }?;

        unsafe { function.call() };

        Ok(())
    }
}
