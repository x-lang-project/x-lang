use super::*;

impl TypeChecker {
    pub(super) fn type_contains_placeholder(value_type: &Type) -> bool {
        value_type.any_nested_type(|inner| matches!(inner, Type::TypeParameter(_)))
    }
}
