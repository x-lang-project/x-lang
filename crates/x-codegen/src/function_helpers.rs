use inkwell::values;

use x_ast::Type;

pub(crate) fn is_generic_placeholder_type(value_type: &Type) -> bool {
    matches!(value_type, Type::TypeParameter(_))
}

pub(crate) fn is_pointer_like_ast_type(value_type: &Type) -> bool {
    matches!(value_type, Type::RawPointer { .. } | Type::Reference { .. })
}

pub(crate) use x_ast::intrinsics::strip_explicit_type_arguments;

pub(crate) fn push_unique_candidate(candidates: &mut Vec<String>, candidate: String) {
    if !candidates.iter().any(|existing| existing == &candidate) {
        candidates.push(candidate);
    }
}

pub(crate) fn is_integer_ast_type(value_type: &Type) -> bool {
    value_type.is_integer()
}

pub(crate) fn function_matches_arity(
    function: values::FunctionValue<'_>,
    arity: Option<usize>,
) -> bool {
    arity.is_none_or(|arity| {
        let required = function.count_params() as usize;

        arity == required || (function.get_type().is_var_arg() && arity > required)
    })
}
