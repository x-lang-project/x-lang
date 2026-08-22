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
