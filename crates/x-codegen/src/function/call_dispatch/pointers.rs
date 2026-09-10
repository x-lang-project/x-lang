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

            _ => Ok(CallEmission::Unsupported),
        }
    }
}
