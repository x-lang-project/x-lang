use super::*;
use x_ast::intrinsics::BuiltinKind;
use x_diagnostics::backend_messages;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
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
