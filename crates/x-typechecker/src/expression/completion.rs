use crate::TypeChecker;
use x_ast::{Expression, Type};
use x_diagnostics::frontend_messages::FrontendMessage;

pub(super) enum CheckedType {
    PreserveAnnotation(Type),
    Annotate(Type),
    ResolveAnnotation(Type),
}
