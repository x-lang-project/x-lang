use super::*;
use x_ast::intrinsics::BuiltinKind;
use x_diagnostics::backend_messages;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_string_slice_call(
        &self,
        call: DispatchCall<'_>,
        arguments: &[Operand],
    ) -> DispatchAttempt<'ctx> {
        match call.identity.kind {
            BuiltinKind::StringBytes => {
                return Err(CodeGenerationError::InvalidMir(
                    backend_messages::slice_view_requires_explicit_identity_lifetime_and_mutability(
                        call.full,
                    ),
                ));
            }
            BuiltinKind::StringView => {}
            _ => return Ok(CallEmission::Unsupported),
        }

        let [receiver] = arguments else {
            return Err(CodeGenerationError::InvalidMir(
                backend_messages::call_requires_one_primitive_string_receiver(call.full),
            ));
        };

        if self.operand_ast_type(receiver) != Some(Type::String) {
            return Err(CodeGenerationError::InvalidMir(
                backend_messages::call_requires_primitive_string_receiver(call.full),
            ));
        }

        self.load_operand_value(receiver).map(CallEmission::Value)
    }
}
