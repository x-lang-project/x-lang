use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::Operand;

pub(super) struct Arguments<'a, 'ctx> {
    compiler: &'a FunctionCompiler<'a, 'ctx>,
    function: &'a str,
    arguments: &'a [Operand],
}

pub(super) struct Argument<'a, 'ctx> {
    compiler: &'a FunctionCompiler<'a, 'ctx>,
    operand: &'a Operand,
}

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn arguments<'a>(
        &'a self,
        function: &'a str,
        arguments: &'a [Operand],
    ) -> Arguments<'a, 'ctx> {
        Arguments {
            compiler: self,
            function,
            arguments,
        }
    }

    pub(super) fn expect_arguments<'b, const N: usize>(
        function: &str,
        arguments: &'b [Operand],
    ) -> Result<&'b [Operand; N], CodeGenerationError> {
        arguments.try_into().map_err(|_| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::function_expects_n_argument_value_got_value(
                    function,
                    N,
                    arguments.len(),
                ),
            )
        })
    }

    pub(super) fn value_argument(
        &self,
        argument: &Operand,
        value_type: &Type,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        self.load_operand_value_with_type(argument, value_type)
    }

    pub(super) fn integer_argument(
        &self,
        argument: &Operand,
        value_type: &Type,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        Ok(self.value_argument(argument, value_type)?.into_int_value())
    }

    pub(super) fn pointer_argument(
        &self,
        argument: &Operand,
        name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        let value = self.load_operand_value(argument)?;

        self.coerce_value_to_pointer(value, name)
    }
}

impl<'a, 'ctx> Arguments<'a, 'ctx> {
    pub(super) fn get(&self, index: usize) -> Result<Argument<'a, 'ctx>, CodeGenerationError> {
        let operand = self.arguments.get(index).ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::value_is_missing_argument_value(self.function, index + 1),
            )
        })?;

        Ok(Argument {
            compiler: self.compiler,
            operand,
        })
    }

    pub(super) fn exact<const N: usize>(
        &self,
    ) -> Result<[Argument<'a, 'ctx>; N], CodeGenerationError> {
        let arguments = FunctionCompiler::expect_arguments::<N>(self.function, self.arguments)?;

        Ok(arguments.each_ref().map(|operand| Argument {
            compiler: self.compiler,
            operand,
        }))
    }
}

impl<'a, 'ctx> Argument<'a, 'ctx> {
    pub(super) fn value(
        &self,
        value_type: &Type,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        self.compiler.value_argument(self.operand, value_type)
    }

    pub(super) fn integer(&self, value_type: &Type) -> Result<IntValue<'ctx>, CodeGenerationError> {
        self.compiler.integer_argument(self.operand, value_type)
    }

    pub(super) fn pointer(&self, name: &str) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        self.compiler.pointer_argument(self.operand, name)
    }

    pub(super) fn operand(&self) -> &'a Operand {
        self.operand
    }
}
