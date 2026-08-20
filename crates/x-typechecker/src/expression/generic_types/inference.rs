use super::*;

pub(in crate::expression) fn infer_declared_type_substitutions(
    checker: &TypeChecker,
    pattern: &Type,
    actual: &Type,
    declared: &[String],
    substitutions: &mut HashMap<String, Type>,
) {
    match (pattern, actual) {
        (Type::TypeParameter(name), _) => {
            substitutions
                .entry(name.clone())
                .or_insert_with(|| actual.clone());
        }
        (Type::Custom(name), _) if declared.contains(name) => {
            substitutions
                .entry(name.clone())
                .or_insert_with(|| actual.clone());
        }
        (
            Type::GenericInstance {
                name: pattern_name,
                type_arguments: pattern_arguments,
            },
            _,
        ) => {
            let actual_arguments = match actual {
                Type::GenericInstance {
                    name,
                    type_arguments,
                } if pattern_name == name => Some(type_arguments),

                _ => None,
            };

            if let Some(actual_arguments) = actual_arguments
                && pattern_arguments.len() == actual_arguments.len()
            {
                for (pattern_argument, actual_argument) in
                    pattern_arguments.iter().zip(actual_arguments)
                {
                    infer_declared_type_substitutions(
                        checker,
                        pattern_argument,
                        actual_argument,
                        declared,
                        substitutions,
                    );
                }
            }
        }
        (
            Type::Reference {
                inner: pattern_inner,
                ..
            },
            Type::Reference {
                inner: actual_inner,
                ..
            },
        )
        | (
            Type::RawPointer {
                inner: pattern_inner,
                ..
            },
            Type::RawPointer {
                inner: actual_inner,
                ..
            },
        )
        | (Type::Array(pattern_inner), Type::Array(actual_inner)) => {
            infer_declared_type_substitutions(
                checker,
                pattern_inner,
                actual_inner,
                declared,
                substitutions,
            );
        }
        (
            Type::FixedSizedArray(pattern_inner, pattern_length),
            Type::FixedSizedArray(actual_inner, actual_length),
        ) if pattern_length == actual_length => {
            infer_declared_type_substitutions(
                checker,
                pattern_inner,
                actual_inner,
                declared,
                substitutions,
            );
        }
        (
            Type::Simd {
                element: pattern_inner,
                lanes: pattern_lanes,
            },
            Type::Simd {
                element: actual_inner,
                lanes: actual_lanes,
            },
        ) if pattern_lanes == actual_lanes => {
            infer_declared_type_substitutions(
                checker,
                pattern_inner,
                actual_inner,
                declared,
                substitutions,
            );
        }
        (Type::Tuple(pattern_items), Type::Tuple(actual_items))
            if pattern_items.len() == actual_items.len() =>
        {
            for (pattern_item, actual_item) in pattern_items.iter().zip(actual_items.iter()) {
                infer_declared_type_substitutions(
                    checker,
                    pattern_item,
                    actual_item,
                    declared,
                    substitutions,
                );
            }
        }
        (
            Type::Function {
                parameters: pattern_parameters,
                return_value: pattern_return,
            },
            Type::Function {
                parameters: actual_parameters,
                return_value: actual_return,
            },
        ) if pattern_parameters.len() == actual_parameters.len() => {
            for (pattern, actual) in pattern_parameters.iter().zip(actual_parameters) {
                infer_declared_type_substitutions(
                    checker,
                    pattern,
                    actual,
                    declared,
                    substitutions,
                );
            }

            infer_declared_type_substitutions(
                checker,
                pattern_return,
                actual_return,
                declared,
                substitutions,
            );
        }
        _ => {}
    }
}
