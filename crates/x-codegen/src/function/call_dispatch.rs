mod pointers;

use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_ast::intrinsics;
use x_diagnostics::backend_messages;
use x_mir::Operand;
use x_mir::ResolvedCall;

#[derive(Debug, Clone, Copy)]
enum CallEmission<'ctx> {
    Value(BasicValueEnum<'ctx>),
    Void,
    Unsupported,
}

type DispatchAttempt<'ctx> = Result<CallEmission<'ctx>, CodeGenerationError>;

#[derive(Debug, Clone, Copy)]
struct DispatchCall<'b> {
    full: &'b str,
    identity: &'b intrinsics::BuiltinCall,
}

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn operand_ast_type(&self, operand: &Operand) -> Option<Type> {
        Some(self.body.operand_type(operand))
    }
}
