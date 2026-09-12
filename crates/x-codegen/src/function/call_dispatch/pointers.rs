use super::*;
use x_diagnostics::backend_messages;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
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
}
