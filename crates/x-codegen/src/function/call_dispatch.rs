mod memory;
mod numeric;
mod pointers;
mod sequences;
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

impl<'ctx> CallEmission<'ctx> {
    fn emitted(value: Option<BasicValueEnum<'ctx>>) -> Self {
        match value {
            Some(value) => Self::Value(value),
            None => Self::Void,
        }
    }
}

type DispatchAttempt<'ctx> = Result<CallEmission<'ctx>, CodeGenerationError>;

#[derive(Debug, Clone, Copy)]
struct DispatchCall<'b> {
    full: &'b str,
    identity: &'b intrinsics::BuiltinCall,
}

impl<'b> DispatchCall<'b> {
    fn from_builtin(call: &'b intrinsics::BuiltinCall) -> Self {
        Self {
            full: &call.name,
            identity: call,
        }
    }
}

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_call_value(
        &self,
        function: &str,
        arguments: &[Operand],
        expected_return_type: Option<&Type>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        if self.codegen.has_exact_source_function(function) {
            return self.emit_resolved_user_call(function, arguments);
        }

        let identity = match self.body.call_targets.get(function) {
            Some(ResolvedCall::Source { .. }) => {
                return self.emit_resolved_user_call(function, arguments);
            }
            Some(ResolvedCall::Builtin(_)) => self
                .body
                .builtin_call(function, arguments, expected_return_type)
                .ok_or_else(|| {
                    CodeGenerationError::InvalidMir(
                        backend_messages::call_function_has_no_matching_typed_signature(function),
                    )
                })?,
            None => {
                return Err(CodeGenerationError::InvalidMir(
                    backend_messages::call_function_has_no_resolved_target(function),
                ));
            }
        };

        let call = DispatchCall::from_builtin(identity);
        use x_ast::intrinsics::BuiltinKind;

        let attempt = match identity.kind {
            BuiltinKind::Pointer(_) => self.emit_pointer_call(call, arguments)?,
            BuiltinKind::StringView | BuiltinKind::StringBytes => {
                self.emit_string_slice_call(call, arguments)?
            }
            BuiltinKind::Print
            | BuiltinKind::Println
            | BuiltinKind::Intrinsic(
                intrinsics::Intrinsic::Float(_)
                | intrinsics::Intrinsic::Overflow(_)
                | intrinsics::Intrinsic::Rotate(_)
                | intrinsics::Intrinsic::Bit(_),
            )
            | BuiltinKind::FromRawParts
            | BuiltinKind::Align(_)
            | BuiltinKind::Number(_)
            | BuiltinKind::Slice(_)
            | BuiltinKind::FillI32 => {
                self.emit_print_or_intrinsic_call(call, arguments, expected_return_type)?
            }
            BuiltinKind::CoreMemory(_)
            | BuiltinKind::HeapAllocate(_)
            | BuiltinKind::Intrinsic(intrinsics::Intrinsic::Layout(_))
            | BuiltinKind::MemSwap
            | BuiltinKind::MemReplace
            | BuiltinKind::AssertNonNull
            | BuiltinKind::AssertAligned => {
                self.emit_layout_allocator_or_memory_call(call, arguments)?
            }
            BuiltinKind::PointerNamespace(_) => self.emit_pointer_namespace_call(call)?,
            BuiltinKind::Intrinsic(intrinsics::Intrinsic::Memory(_)) => {
                self.emit_memory_intrinsic(identity, arguments)?;

                CallEmission::Void
            }
            BuiltinKind::Intrinsic(intrinsics::Intrinsic::Control(_)) => {
                return Err(CodeGenerationError::InvalidMir(
                    backend_messages::abort_must_be_a_non_returning_mir_terminator(),
                ));
            }
        };

        match attempt {
            CallEmission::Value(value) => Ok(Some(value)),
            CallEmission::Void => Ok(None),
            CallEmission::Unsupported => Err(CodeGenerationError::InvalidMir(
                backend_messages::resolved_intrinsic_value_could_not_emit_function(
                    identity.kind,
                    function,
                ),
            )),
        }
    }

    fn emit_resolved_user_call(
        &self,
        callee: &str,
        arguments: &[Operand],
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        let target_function = self.resolve_callee(callee, Some(arguments.len()))?;

        let function_parameters = target_function.get_params();

        let call_arguments = arguments
            .iter()
            .enumerate()
            .map(|(index, argument)| {
                let mut element = self.load_operand_value(argument)?;

                if let Some(parameter) = function_parameters.get(index) {
                    element = self.normalize_call_argument(
                        crate::function::numeric::TypedValue {
                            value: element,
                            value_type: &self.body.operand_type(argument),
                        },
                        *parameter,
                        callee,
                        index,
                    )?;
                }

                Ok(element.into())
            })
            .collect::<Result<Vec<_>, CodeGenerationError>>()?;

        let call_site = self.call(target_function, &call_arguments, "call_expr")?;

        Ok(call_site.try_as_basic_value().basic())
    }

    pub(super) fn operand_ast_type(&self, operand: &Operand) -> Option<Type> {
        Some(self.body.operand_type(operand))
    }
}
