use crate::TypeChecker;
use crate::expression::CheckedType;
use x_ast::{CallTarget, Expression, ExpressionKind, Type};

pub(in crate::expression) enum CheckedMethod {
    Builtin(Type),
    DirectCall {
        return_type: Type,
        target: CallTarget,
        arguments: Vec<Expression>,
    },
}
