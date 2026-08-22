use std::mem;

use super::DeclaredMethodCall;
use crate::expression::CheckedType;
use crate::{TypeChecker, errors::TypeError};
use x_ast::{CallTarget, Expression, ExpressionKind, Type};

pub(in crate::expression) enum CheckedMethod {
    Builtin(Type),
    DirectCall {
        return_type: Type,
        target: CallTarget,
        arguments: Vec<Expression>,
    },
}

impl CheckedMethod {
    pub(in crate::expression) fn apply_to(self, kind: &mut ExpressionKind) -> CheckedType {
        match self {
            Self::Builtin(return_type) => CheckedType::Annotate(return_type),
            Self::DirectCall {
                return_type,
                target,
                arguments,
            } => {
                *kind = ExpressionKind::FunctionCall { target, arguments };

                CheckedType::ResolveAnnotation(return_type)
            }
        }
    }
}

impl TypeChecker {
    pub(super) fn lower_declared_method_call(
        &mut self,
        object: &Expression,
        target: &CallTarget,
        current_arguments: &mut Vec<Expression>,
        call: DeclaredMethodCall,
    ) -> Result<CheckedMethod, TypeError> {
        let receiver = self.method_receiver_argument(object, &call.receiver_parameter)?;
        let mut arguments = Vec::with_capacity(current_arguments.len() + 1);
        arguments.push(receiver);
        arguments.extend(mem::take(current_arguments));
        let mut target = target.clone();
        target.resolve_to(call.callee);

        Ok(CheckedMethod::DirectCall {
            return_type: call.return_type,
            target,
            arguments,
        })
    }
}
