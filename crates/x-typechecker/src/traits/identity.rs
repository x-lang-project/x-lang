use super::*;

pub(super) fn qualified_names_match(left: &str, right: &str) -> bool {
    left == right
        || ((!left.contains("::") || !right.contains("::"))
            && left.rsplit("::").next() == right.rsplit("::").next())
}

pub(super) fn trait_generic_parameters(trait_definition: &TraitDefinition) -> Vec<String> {
    trait_definition
        .generic_parameters
        .iter()
        .filter_map(|parameter| match parameter {
            x_ast::GenericParameter::Type { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect()
}

pub(super) fn trait_generic_arguments_for_impl(trait_name: &str) -> Vec<Type> {
    let Some(arguments) = angle_contents(trait_name) else {
        return Vec::new();
    };

    split_top_level_commas(arguments)
        .into_iter()
        .map(|argument| {
            let argument = argument.trim();

            type_from_trait_argument(argument)
        })
        .collect()
}

pub(super) fn angle_contents(text: &str) -> Option<&str> {
    let start = text.find('<')?;
    let end = text.rfind('>')?;

    start
        .checked_add(1)
        .and_then(|contents_start| text.get(contents_start..end))
}

pub(super) fn type_from_trait_argument(text: &str) -> Type {
    x_ast::type_from_name(text)
}

pub(super) fn trait_method_has_default_body(statement: &Statement) -> bool {
    matches!(statement, Statement::Function { body: Some(_), .. })
}

pub(super) fn substitute_associated_self_types(
    value_type: &mut Type,
    associated_types: &BTreeMap<String, Type>,
) {
    if let Type::Custom(name) | Type::TypeParameter(name) | Type::GenericInstance { name, .. } =
        value_type
        && let Some(assoc_name) = name.strip_prefix("Self::")
        && let Some(assoc_type) = associated_types.get(assoc_name)
    {
        *value_type = assoc_type.clone();
        return;
    }

    match value_type {
        Type::GenericInstance {
            type_arguments: items,
            ..
        }
        | Type::Tuple(items) => {
            for item in items {
                substitute_associated_self_types(item, associated_types);
            }
        }
        Type::Array(inner)
        | Type::FixedSizedArray(inner, _)
        | Type::Simd { element: inner, .. }
        | Type::RawPointer { inner, .. }
        | Type::Reference { inner, .. } => {
            substitute_associated_self_types(inner, associated_types)
        }
        Type::Function {
            parameters,
            return_value,
        } => {
            for parameter in parameters {
                substitute_associated_self_types(parameter, associated_types);
            }

            substitute_associated_self_types(return_value, associated_types);
        }
        _ => {}
    }
}
