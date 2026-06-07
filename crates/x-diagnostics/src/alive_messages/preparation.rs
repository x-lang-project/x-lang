use super::*;

pub fn example_enumeration(error: impl error::Error + 'static) -> AliveError {
    AliveError::caused(
        AliveFailureKind::ExampleEnumeration,
        Error::new(format!("Failed to enumerate examples: {error}")),
        error,
    )
}

pub fn compile_original(
    source_path: impl Display,
    error: impl error::Error + 'static,
) -> AliveError {
    AliveError::caused(
        AliveFailureKind::CompileOriginal,
        Error::new(format!(
            "Failed to compile keyword-enabled source {source_path}: {error}"
        )),
        error,
    )
}

pub fn unclosed_block(start: impl Display) -> AliveError {
    AliveError::new(
        AliveFailureKind::UnclosedBlock,
        Error::new(format!("Unclosed comparison block at byte {start}")),
    )
}

pub fn invalid_token_boundary() -> AliveError {
    AliveError::new(
        AliveFailureKind::InvalidTokenBoundary,
        Error::new("Invalid comparison token boundary"),
    )
}

pub fn invalid_attribute_boundary() -> AliveError {
    AliveError::new(
        AliveFailureKind::InvalidAttributeBoundary,
        Error::new("Invalid attribute boundary"),
    )
}

pub fn invalid_attribute_name_boundary() -> AliveError {
    AliveError::new(
        AliveFailureKind::InvalidAttributeNameBoundary,
        Error::new("Invalid attribute name boundary"),
    )
}

pub fn invalid_edit_boundary() -> AliveError {
    AliveError::new(
        AliveFailureKind::InvalidEditBoundary,
        Error::new("Invalid comparison edit boundary"),
    )
}

pub fn examples_missing(path: impl Display) -> AliveError {
    AliveError::new(
        AliveFailureKind::ExamplesMissing,
        Error::new(format!("Examples directory not found: {path}")),
    )
}
