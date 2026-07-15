use super::*;

pub(super) fn validate_name(
    name: &str,
    path: impl FnOnce() -> String,
) -> Result<(), HirValidationError> {
    if name.trim().is_empty() {
        Err(HirValidationError::EmptySymbol { path: path() })
    } else {
        Ok(())
    }
}

pub(super) fn validate_type(
    value_type: &Type,
    path: impl FnOnce() -> String,
) -> Result<(), HirValidationError> {
    if contains_unresolved_type(value_type) {
        Err(HirValidationError::UnresolvedType {
            path: path(),
            value_type: value_type.clone(),
        })
    } else {
        Ok(())
    }
}

pub(super) fn contains_unresolved_type(value_type: &Type) -> bool {
    value_type.any_nested_type(|inner| matches!(inner, Type::Unknown | Type::Infer(_)))
}

pub(super) fn validate_call_target(
    target: &x_ast::CallTarget,
    path: &str,
) -> Result<(), HirValidationError> {
    if target.path.segments.is_empty() {
        validate_name("", || path.to_owned())?;
    }

    for segment in &target.path.segments {
        validate_name(&segment.name, || path.to_owned())?;
    }

    if let Some(name) = target.resolved_name() {
        validate_name(name, || path.to_owned())?;
    }

    if let Some(x_ast::CallResolution::Builtin { type_arguments, .. }) = &target.resolution {
        for argument in type_arguments {
            validate_type(argument, || path.to_owned())?;
        }
    }

    Ok(())
}
