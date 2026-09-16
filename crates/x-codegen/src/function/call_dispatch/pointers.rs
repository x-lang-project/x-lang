use super::*;
use x_diagnostics::backend_messages;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_pointer_call(
        &self,
        call: DispatchCall<'_>,
        arguments: &[Operand],
    ) -> DispatchAttempt<'ctx> {
        use intrinsics::{BuiltinKind, PointerOperation};

        let BuiltinKind::Pointer(operation) = call.identity.kind else {
            return Ok(CallEmission::Unsupported);
        };

        let value = match operation {
            PointerOperation::IntrinsicOffset => {
                Some(self.emit_pointer_offset_intrinsic(call.identity, arguments)?)
            }
            PointerOperation::Add | PointerOperation::Subtract | PointerOperation::Offset => {
                Some(self.emit_pointer_step_method(call.identity, arguments)?)
            }
            PointerOperation::AsReference | PointerOperation::AsMutableReference => {
                Some(self.emit_pointer_option_method(call.full, arguments)?)
            }
            PointerOperation::Read
            | PointerOperation::ReadUnaligned
            | PointerOperation::ReadVolatile => {
                Some(self.emit_pointer_read_intrinsic(call.identity, arguments)?)
            }
            PointerOperation::Write
            | PointerOperation::WriteUnaligned
            | PointerOperation::WriteVolatile => {
                self.emit_pointer_write_intrinsic(call.identity, arguments)?;

                None
            }
            PointerOperation::Replace => {
                Some(self.emit_pointer_replace_intrinsic(call.identity, arguments)?)
            }
            PointerOperation::Swap => {
                self.emit_pointer_swap_intrinsic(call.identity, arguments)?;

                None
            }
            PointerOperation::Copy | PointerOperation::CopyNonoverlapping => {
                self.emit_pointer_copy_intrinsic(call.identity, arguments)?;

                None
            }
            PointerOperation::WriteBytes => {
                self.emit_pointer_write_bytes_intrinsic(call.identity, arguments)?;

                None
            }
            PointerOperation::WithAddress => {
                Some(self.emit_pointer_with_address_intrinsic(call.full, arguments)?)
            }
            PointerOperation::FromAddress => {
                Some(self.emit_pointer_from_address_intrinsic(call.full, arguments)?)
            }
            PointerOperation::Address => {
                Some(self.emit_pointer_address_intrinsic(call.full, arguments)?)
            }
            PointerOperation::IsNull => Some(self.emit_pointer_is_null(arguments, "ptr_is_null")?),
        };

        Ok(CallEmission::emitted(value))
    }

    pub(super) fn emit_pointer_namespace_call(
        &self,
        call: DispatchCall<'_>,
    ) -> DispatchAttempt<'ctx> {
        use intrinsics::{BuiltinKind, PointerConstructor};

        match call.identity.kind {
            BuiltinKind::PointerNamespace(PointerConstructor::Null) => {
                Ok(CallEmission::Value(self.pointer_type().const_null().into()))
            }
            BuiltinKind::PointerNamespace(PointerConstructor::Dangling) => {
                let pointee = call
                    .identity
                    .type_argument
                    .as_ref()
                    .or_else(|| match call.identity.return_type.as_ref() {
                        Some(Type::RawPointer { inner, .. }) => Some(inner.as_ref()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        CodeGenerationError::InvalidMir(
                            backend_messages::dangling_requires_resolved_pointer_element(),
                        )
                    })?;
                let (_, alignment) = self.codegen.resolve_type_layout(pointee)?;
                let address = self.usize_type().const_int(alignment, false);

                self.integer_to_pointer(address, self.pointer_type(), "ptr_helper")
                    .map(|pointer| CallEmission::Value(pointer.into()))
            }
            _ => Ok(CallEmission::Unsupported),
        }
    }

    pub(super) fn emit_pointer_is_null(
        &self,
        arguments: &[Operand],
        cmp_name: &str,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let [pointer] = self.arguments(cmp_name, arguments).exact()?;
        let pointer = self.load_operand_value(pointer.operand())?;
        let BasicValueEnum::PointerValue(pointer) = pointer else {
            return Err(CodeGenerationError::InvalidMir(
                backend_messages::is_null_requires_typed_pointer(),
            ));
        };

        self.b()
            .build_is_null(pointer, cmp_name)
            .map(|value| value.into())
            .map_err(Into::into)
    }
}
