mod address;
mod memory;
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
    pub(super) fn emit_pointer_offset_intrinsic(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let callee = call.name.as_str();
        let [base_argument, offset_argument] = self.arguments(callee, arguments).exact()?;

        let base = self.load_operand_value(base_argument.operand())?;
        let offset = offset_argument.value(&Type::ISize)?;

        let pointee = self.pointer_memory_element_type(call, arguments, 0)?;

        self.emit_pointer_arithmetic(
            &Operator::Add,
            base,
            offset,
            crate::function::coercions::PointerArithmeticTypes {
                pointee: Some(&pointee),
                offset: &Type::ISize,
            },
        )?
        .ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::function_requires_pointer_and_integer_operands(callee),
            )
        })
    }

    pub(super) fn emit_pointer_step_method(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let function = call.name.as_str();
        let [pointer_argument, count_argument] = self.arguments(function, arguments).exact()?;

        let pointer = pointer_argument.pointer("ptr_step_addr")?;
        let mut count = count_argument.integer(&Type::ISize)?;

        if call.kind
            == x_ast::intrinsics::BuiltinKind::Pointer(
                x_ast::intrinsics::PointerOperation::Subtract,
            )
        {
            count = self.integer_neg(count, "ptr_step_neg_count")?;
        }

        let element_type = self
            .pointer_intrinsic_pointee_type(call, Some(pointer_argument.operand()))
            .ok_or_else(|| {
                CodeGenerationError::InvalidMir(
                    backend_messages::call_requires_resolved_pointer_element(function),
                )
            })?;
        let llvm_element_type = self.codegen.resolve_llvm_type(&element_type)?;

        self.gep(llvm_element_type, pointer, count, "ptr_element_offset")
            .map(|pointer| pointer.into())
    }

    pub(super) fn emit_pointer_option_method(
        &self,
        function: &str,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let [argument] = self.arguments(function, arguments).exact()?;

        argument
            .pointer("ptr_option_value")
            .map(|pointer| pointer.into())
    }

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
