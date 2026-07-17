use super::MirValidationError;
use x_ast::Type;

pub(super) fn validate_type(
    value_type: &Type,
    context: &dyn Fn() -> String,
) -> Result<(), MirValidationError> {
    match value_type {
        Type::Unknown | Type::Infer(_) | Type::TypeParameter(_) => {
            Err(MirValidationError::UnresolvedType {
                context: context(),
                value_type: value_type.clone(),
            })
        }
        Type::Array(inner)
        | Type::FixedSizedArray(inner, _)
        | Type::Simd { element: inner, .. }
        | Type::RawPointer { inner, .. }
        | Type::Reference { inner, .. } => validate_type(inner, context),
        Type::Tuple(items)
        | Type::GenericInstance {
            type_arguments: items,
            ..
        } => {
            for item in items {
                validate_type(item, context)?;
            }

            Ok(())
        }
        Type::Function {
            parameters,
            return_value,
        } => {
            for parameter in parameters {
                validate_type(parameter, context)?;
            }

            validate_type(return_value, context)
        }
        _ => Ok(()),
    }
}
