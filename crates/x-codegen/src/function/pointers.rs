mod address;

use inkwell::values::{BasicValueEnum, PointerValue};

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use crate::function_helpers::is_generic_placeholder_type;
use x_ast::Operator;
use x_ast::Type;
use x_ast::intrinsics::BuiltinCall;
use x_diagnostics::backend_messages;
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn pointer_memory_element_type(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
        pointer_argument_index: usize,
    ) -> Result<Type, CodeGenerationError> {
        self.pointer_intrinsic_pointee_type(call, arguments.get(pointer_argument_index))
            .ok_or_else(|| {
                CodeGenerationError::InvalidMir(
                    backend_messages::call_requires_resolved_pointer_element(&call.name),
                )
            })
    }

    pub(super) fn pointer_intrinsic_pointee_type(
        &self,
        call: &BuiltinCall,
        operand: Option<&Operand>,
    ) -> Option<Type> {
        call.type_argument
            .clone()
            .filter(|value_type| !is_generic_placeholder_type(value_type))
            .or_else(|| operand.and_then(|argument| self.pointer_operand_pointee_type(argument)))
    }

    pub(super) fn pointer_operand_pointee_type(&self, operand: &Operand) -> Option<Type> {
        match self.operand_ast_type(operand)? {
            Type::RawPointer { inner, .. } | Type::Reference { inner, .. } => Some(*inner),
            Type::Infer(_) | Type::Unknown | Type::Void => None,
            _ => None,
        }
    }
}
