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

impl TypeChecker {
    pub(super) fn check_string_method(
        receiver_base: Type,
        method_base: String,
        arguments: &mut [Expression],
    ) -> Result<Type, TypeError> {
        if !arguments.is_empty() {
            return Err(TypeError::ArityMismatch {
                expected: 0,
                got: arguments.len(),
                symbol_name: format!("str::{method_base}"),
            });
        }

        match method_base.as_str() {
            "len" => Ok(Type::USize),
            "as_str" => Ok(Type::String),
            _ => Err(TypeError::UnresolvedProperty {
                field: method_base,
                value_type: receiver_base,
            }),
        }
    }
}
