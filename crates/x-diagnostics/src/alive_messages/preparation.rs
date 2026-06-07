use super::*;

pub fn directory_create(directory: impl Display, error: impl error::Error + 'static) -> AliveError {
    AliveError::caused(
        AliveFailureKind::DirectoryCreate,
        Error::new(format!("Failed to create dir {directory}: {error}")),
        error,
    )
}

pub fn file_write(path: impl Display, error: impl error::Error + 'static) -> AliveError {
    AliveError::caused(
        AliveFailureKind::FileWrite,
        Error::new(format!("Failed to write {path}: {error}")),
        error,
    )
}

pub fn ir_normalizer(error: impl error::Error + 'static) -> AliveError {
    AliveError::caused(
        AliveFailureKind::IrNormalizer,
        Error::new(format!("failed to construct Alive2 IR normaliser: {error}")),
        error,
    )
}

pub fn borrow_normalizer(error: impl error::Error + 'static) -> AliveError {
    AliveError::caused(
        AliveFailureKind::BorrowNormalizer,
        Error::new(format!(
            "failed to construct borrow attribute normaliser: {error}"
        )),
        error,
    )
}

pub fn attribute_normalizer(error: impl error::Error + 'static) -> AliveError {
    AliveError::caused(
        AliveFailureKind::AttributeNormalizer,
        Error::new(format!(
            "failed to construct Alive2 attribute normaliser: {error}"
        )),
        error,
    )
}

pub fn logging(error: impl error::Error + 'static) -> AliveError {
    AliveError::caused(
        AliveFailureKind::Logging,
        Error::new(format!("failed to initialise logging: {error}")),
        error,
    )
}

pub fn optimizer_probe(output: impl Display) -> AliveError {
    AliveError::new(
        AliveFailureKind::OptimizerProbe,
        Error::new(format!("LLVM O0-to-O3 optimiser probe failed:\n{output}")).with_code("A0004"),
    )
}

pub fn example_enumeration(error: impl error::Error + 'static) -> AliveError {
    AliveError::caused(
        AliveFailureKind::ExampleEnumeration,
        Error::new(format!("Failed to enumerate examples: {error}")),
        error,
    )
}

pub fn file_read(path: impl Display, error: impl error::Error + 'static) -> AliveError {
    AliveError::caused(
        AliveFailureKind::FileRead,
        Error::new(format!("Failed to read {path}: {error}")),
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

pub fn parse_comparison(
    source_path: impl Display,
    failure: impl error::Error + 'static,
) -> AliveError {
    AliveError::caused(
        AliveFailureKind::ParseComparison,
        Error::new(format!(
            "Failed to parse stripped comparison {source_path}: {failure}"
        )),
        failure,
    )
}

pub fn compile_comparison(
    keyword_tag: impl Display,
    source_path: impl Display,
    error: impl error::Error + 'static,
) -> AliveError {
    AliveError::caused(
        AliveFailureKind::CompileComparison,
        Error::new(format!(
            "Failed to compile source without {keyword_tag} at {source_path}: {error}"
        )),
        error,
    )
}

pub fn verification(source_path: impl Display) -> AliveError {
    AliveError::new(
        AliveFailureKind::Verification,
        Error::new(format!(
            "Alive2 verification failed for {source_path}. Re-run with --keep-going to continue."
        )),
    )
}

pub fn comparisons_failed(failed: impl Display) -> AliveError {
    AliveError::new(
        AliveFailureKind::ComparisonsFailed,
        Error::new(format!("{failed} comparison(s) failed")),
    )
}

pub fn unclosed_block(start: impl Display) -> AliveError {
    AliveError::new(
        AliveFailureKind::UnclosedBlock,
        Error::new(format!("Unclosed comparison block at byte {start}")),
    )
}

pub fn invalid_transformed_source(error: impl error::Error + 'static) -> AliveError {
    AliveError::caused(
        AliveFailureKind::InvalidTransformedSource,
        Error::new(format!("Invalid transformed source: {error}")),
        error,
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

pub fn tool_missing() -> AliveError {
    AliveError::new(
        AliveFailureKind::ToolMissing,
        Error::new(
            "alive-tv was not found. Pass --alive-tv, set X_ALIVE_TV/ALIVE_TV, install it on PATH, or use --emit-only.",
        ),
    )
}

pub fn no_comparisons(path: impl Display) -> AliveError {
    AliveError::new(
        AliveFailureKind::NoComparisons,
        Error::new(format!("No .x files selected in {path}")),
    )
}
