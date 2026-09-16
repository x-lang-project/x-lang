use inkwell::attributes::Attribute;
use inkwell::values::LLVMTailCallKind::LLVMTailCallKindMustTail;

use super::super::FunctionCompiler;
use super::{CallContinuation, TAIL_ABI_ATTRIBUTES, tail_abi_attributes_match, tail_abi_locations};
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::Operand;

impl FunctionCompiler<'_, '_> {
    pub(super) fn compile_call_terminator(
        &self,
        callee: &str,
        arguments: &[Operand],
        continuation: CallContinuation<'_>,
    ) -> Result<(), CodeGenerationError> {
        let CallContinuation {
            destination,
            target,
        } = continuation;
        let expected_ast_type = self.get_place_type(destination);

        if let Some(return_value) =
            self.emit_call_value(callee, arguments, Some(&expected_ast_type))?
        {
            let destination_pointer = self.resolve_place_pointer(destination)?;
            let inferred_type = self
                .infer_call_result_type(callee, arguments, &expected_ast_type)
                .unwrap_or_else(|| expected_ast_type.clone());
            let expected_type = self.codegen.resolve_llvm_type(&inferred_type)?;
            let normalized_return = self.normalize_value_for_store(
                crate::function::numeric::TypedValue {
                    value: return_value,
                    value_type: &self
                        .infer_call_result_type(callee, arguments, &Type::Unknown)
                        .unwrap_or(Type::Unknown),
                },
                &inferred_type,
                expected_type,
                "call_ret_type_adjust",
            )?;
            self.store(destination_pointer, normalized_return)?;
        }

        let next_basic_block = self.llvm_blocks.get(&target.0).ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::missing_resume_block_basic_block_value(target.0),
            )
        })?;
        self.br(*next_basic_block)?;

        Ok(())
    }

    pub(super) fn compile_tail_call(
        &self,
        function: &str,
        arguments: &[Operand],
    ) -> Result<(), CodeGenerationError> {
        let target_function = match self.resolve_callee(function, Some(arguments.len())) {
            Ok(target_function) => target_function,
            Err(CodeGenerationError::MissingCalleeSymbol(_)) => {
                return Err(CodeGenerationError::MissingCalleeSymbol(function.into()));
            }
            Err(error) => return Err(error),
        };

        let caller = self.current_function()?;

        if caller.get_type() != target_function.get_type()
            || caller.get_call_conventions() != target_function.get_call_conventions()
            || !tail_abi_attributes_match(caller, target_function)
        {
            return Err(CodeGenerationError::InvalidMir(
                backend_messages::become_requires_identical_abi(self.function_name, function),
            ));
        }

        let function_parameters = target_function.get_params();
        let call_arguments: Vec<_> = arguments
            .iter()
            .enumerate()
            .map(|(index, argument)| {
                let mut element = self.load_operand_value(argument)?;

                if let Some(parameter) = function_parameters.get(index)
                    && element.is_int_value()
                    && parameter.is_int_value()
                {
                    let (v_integer, p_integer) =
                        (element.into_int_value(), parameter.into_int_value());

                    if v_integer.get_type().get_bit_width() != p_integer.get_type().get_bit_width()
                    {
                        element = self
                            .integer_cast_signed(
                                v_integer,
                                p_integer.get_type(),
                                super::super::numeric::is_signed_integer_type(
                                    &self.body.operand_type(argument),
                                ),
                                "tail_param_cast",
                            )?
                            .into();
                    }
                }

                Ok(element.into())
            })
            .collect::<Result<Vec<_>, CodeGenerationError>>()?;

        let call_site = self.call(target_function, &call_arguments, "become_tail")?;
        // X's become requires constant-stack tail calls.
        call_site.set_call_convention(target_function.get_call_conventions());
        for location in tail_abi_locations(target_function.count_params()) {
            for name in TAIL_ABI_ATTRIBUTES {
                let kind = Attribute::get_named_enum_kind_id(name);

                if kind != 0
                    && let Some(attribute) = target_function.get_enum_attribute(location, kind)
                {
                    call_site.add_attribute(location, attribute);
                }
            }
        }

        call_site.set_tail_call_kind(LLVMTailCallKindMustTail);

        let return_type = self.return_type_or_void();

        if *return_type == Type::Void {
            self.return_value(None)?;
        } else if let Some(value) = call_site.try_as_basic_value().basic() {
            self.return_value(Some(&value))?;
        } else {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::become_function_returned_void_for_non_void_function(function),
            ));
        }

        Ok(())
    }
}
