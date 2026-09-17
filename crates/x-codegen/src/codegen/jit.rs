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

    pub fn get_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    pub fn jit_execute(&self) -> Result<f64, CodeGenerationError> {
        let function =
            self.module
                .get_function(x_ast::ENTRY_POINT_NAME)
                .or_else(|| {
                    self.module
                        .get_functions()
                        .find(|callee| callee.count_basic_blocks() > 0)
                })
                .ok_or_else(|| {
                    CodeGenerationError::JitExecutionFailure(backend_messages::no_executable_function_entries_located_inside_active_target_module_context())
                })?;

        let name = function.get_name().to_str().map_err(|_| {
            CodeGenerationError::JitExecutionFailure(
                backend_messages::function_name_is_not_valid_utf(),
            )
        })?;

        let signature = function.get_type();

        if signature.count_param_types() != 0
            || signature.is_var_arg()
            || function.get_call_conventions() != 0
            || function.count_basic_blocks() == 0
        {
            return Err(CodeGenerationError::JitExecutionFailure(
                backend_messages::jit_entry_name_must_have_a_body_and_a_non_variadic_zero(name),
            ));
        }

        self.module.verify().map_err(|error| {
            CodeGenerationError::JitExecutionFailure(backend_messages::invalid_jit_module_error(
                error,
            ))
        })?;
        let return_type = signature.get_return_type();

        unsafe {
            match return_type {
                None => {
                    self.call_jit_function_void(name)?;

                    Ok(0.0)
                }

                Some(value_type) if value_type == self.context.i32_type().into() => {
                    let callee = self.jit_function::<unsafe extern "C" fn() -> i32>(name)?;

                    Ok(callee.call() as f64)
                }

                Some(value_type) if value_type == self.context.i64_type().into() => {
                    let callee = self.jit_function::<unsafe extern "C" fn() -> i64>(name)?;

                    Ok(callee.call() as f64)
                }

                Some(value_type) if value_type == self.context.f64_type().into() => {
                    let callee = self.jit_function::<unsafe extern "C" fn() -> f64>(name)?;

                    Ok(callee.call())
                }

                Some(value_type) if value_type == self.context.f32_type().into() => {
                    let callee = self.jit_function::<unsafe extern "C" fn() -> f32>(name)?;

                    Ok(callee.call() as f64)
                }

                Some(other) => Err(CodeGenerationError::JitExecutionFailure(
                    backend_messages::unsupported_jit_return_type_for_name_other(name, other),
                )),
            }
        }
    }
}
