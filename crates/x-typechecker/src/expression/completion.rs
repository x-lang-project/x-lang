use crate::{TypeChecker, errors::TypeError};
use x_ast::{Expression, Type};
use x_diagnostics::frontend_messages::FrontendMessage;

pub(super) enum CheckedType {
    PreserveAnnotation(Type),
    Annotate(Type),
    ResolveAnnotation(Type),
}

impl CheckedType {
    pub(super) fn complete(
        self,
        checker: &TypeChecker,
        expression: &mut Expression,
    ) -> Result<Type, TypeError> {
        match self {
            Self::PreserveAnnotation(result) => Ok(result),
            Self::Annotate(result) => {
                expression.value_type = result.clone();

                Ok(result)
            }
            Self::ResolveAnnotation(result) => {
                let annotation = checker.resolve(&result);

                if annotation == Type::Unknown {
                    return Err(TypeError::lookup(
                        FrontendMessage::UnresolvedExpressionType {
                            kind: format!("{:?}", expression.kind),
                        },
                    ));
                }

                expression.value_type = annotation;

                Ok(result)
            }
        }
    }
}
