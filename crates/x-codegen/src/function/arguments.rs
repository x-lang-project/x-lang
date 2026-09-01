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
}

impl<'a, 'ctx> Argument<'a, 'ctx> {
    pub(super) fn operand(&self) -> &'a Operand {
        self.operand
    }
}
