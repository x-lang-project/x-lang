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
    pub(super) fn operand(&self) -> &'a Operand {
        self.operand
    }
}
