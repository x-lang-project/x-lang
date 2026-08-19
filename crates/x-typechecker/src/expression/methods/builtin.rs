use super::*;
use x_ast::intrinsics::SliceOperation;

fn array_method(name: &str) -> Option<SliceOperation> {
    if matches!(
        name,
        "len"
            | "is_empty"
            | "has_duplicate_by"
            | "unique_by"
            | "first"
            | "last"
            | "get"
            | "contains"
            | "sum"
    ) {
        SliceOperation::resolve(name)
    } else {
        None
    }
}
