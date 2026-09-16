use super::*;
use x_ast::intrinsics::BuiltinKind;
use x_diagnostics::backend_messages;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_print_or_intrinsic_call(
        &self,
        call: DispatchCall<'_>,
        arguments: &[Operand],
        expected_return_type: Option<&Type>,
    ) -> DispatchAttempt<'ctx> {
        let value = match call.identity.kind {
            BuiltinKind::Print => {
                self.emit_print(arguments)?;

                return Self::print_call_result(expected_return_type);
            }
            BuiltinKind::Println => {
                self.emit_println(arguments)?;

                return Self::print_call_result(expected_return_type);
            }
            BuiltinKind::Intrinsic(intrinsic) => match intrinsic {
                intrinsics::Intrinsic::Overflow(intrinsic) => {
                    self.emit_resolved_overflow_intrinsic(call.full, arguments, intrinsic)?
                }
                intrinsics::Intrinsic::Rotate(direction) => {
                    self.emit_resolved_rotate_intrinsic(call.full, arguments, direction)?
                }
                intrinsics::Intrinsic::Float(intrinsic) => {
                    self.emit_resolved_float_intrinsic(call.full, arguments, intrinsic)?
                }
                intrinsics::Intrinsic::Bit(intrinsic) => {
                    self.emit_resolved_bit_intrinsic(call.full, arguments, intrinsic)?
                }
                intrinsics::Intrinsic::Control(_)
                | intrinsics::Intrinsic::Memory(_)
                | intrinsics::Intrinsic::Layout(_) => return Ok(CallEmission::Unsupported),
            },
            BuiltinKind::FromRawParts => self.emit_from_raw_parts(call.full, arguments)?,
            BuiltinKind::Align(_) => {
                self.emit_resolved_align_intrinsic(call.identity, arguments)?
            }
            BuiltinKind::Number(_) => {
                self.emit_resolved_number_intrinsic(call.identity, arguments)?
            }
            BuiltinKind::Slice(_) => self.emit_resolved_slice_method(call.identity, arguments)?,
            BuiltinKind::FillI32 => {
                self.emit_fill_i32_helper(call.full, arguments)?;

                return Ok(CallEmission::Void);
            }
            _ => return Ok(CallEmission::Unsupported),
        };

        Ok(CallEmission::Value(value))
    }

    pub(super) fn print_call_result(expected_return_type: Option<&Type>) -> DispatchAttempt<'ctx> {
        if matches!(
            expected_return_type,
            None | Some(Type::Void | Type::Unknown | Type::Infer(_))
        ) {
            Ok(CallEmission::Void)
        } else {
            Err(CodeGenerationError::InvalidMir(
                backend_messages::print_requires_declared_value_function(),
            ))
        }
    }
}
