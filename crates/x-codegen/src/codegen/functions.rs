mod body;
mod locals;
mod signatures;

use inkwell::values::FunctionValue;

use super::CodeGeneration;
use crate::codegen::items::{FunctionDeclaration, MirProgramItems};
use crate::errors::CodeGenerationError;
use x_mir::{MirBody, MirProgram};

impl<'ctx> CodeGeneration<'ctx> {
    pub(super) fn declare_function_signatures(
        &self,
        mir_program: &MirProgram,
    ) -> Result<(), CodeGenerationError> {
        for function in mir_program.functions() {
            let (parameters, return_value) = Self::signature_types(
                function.body,
                function.parameter_types,
                function.return_type,
            );
            self.declare_function_signature(FunctionDeclaration {
                parameter_types: &parameters,
                return_type: &return_value,
                ..function
            })?;
            self.internalize_private_definition(function.name, function.body, function.meta);
        }

        Ok(())
    }

    pub(super) fn compile_function_bodies(
        &self,
        mir_program: &MirProgram,
    ) -> Result<(), CodeGenerationError> {
        for (name, body) in mir_program.function_bodies() {
            self.compile_function_body(name, body)?;
        }

        self.compile_missing_function_bodies(mir_program)?;

        Ok(())
    }

    fn compile_missing_function_bodies(
        &self,
        mir_program: &MirProgram,
    ) -> Result<(), CodeGenerationError> {
        for (name, body) in mir_program.function_bodies() {
            let llvm_name = self.llvm_function_symbol(name);
            let Some(function) = self.module.get_function(llvm_name) else {
                continue;
            };

            if function.count_basic_blocks() == 0 {
                self.compile_function_body(name, body)?;
            }
        }

        Ok(())
    }

    pub fn compile_function_body(
        &self,
        function_name: &str,
        body: &MirBody,
    ) -> Result<FunctionValue<'ctx>, CodeGenerationError> {
        let block_order = Self::sorted_basic_block_identifiers(body);
        let (function, mut compiler) =
            self.prepare_function_compiler(function_name, body, &block_order)?;

        compiler.compile_body(&block_order)?;
        self.finalize_function_body(function, function_name)?;

        Ok(function)
    }
}
