use super::*;

impl TypeChecker {
    pub fn reference_compatible(&self, expected: &Type, actual: &Type) -> bool {
        self.types_compatible(expected, actual)
    }

    pub fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }

        match (expected, actual) {
            (Type::TypeParameter(_), _) | (_, Type::TypeParameter(_)) => true,
            (
                Type::GenericInstance {
                    name: expected_name,
                    type_arguments: expected_arguments,
                },
                Type::GenericInstance {
                    name: actual_name,
                    type_arguments: actual_arguments,
                },
            ) => {
                (expected_name == actual_name)
                    && expected_arguments.len() == actual_arguments.len()
                    && expected_arguments.iter().zip(actual_arguments).all(
                        |(expected_argument, actual_argument)| {
                            expected_argument == &Type::Unknown
                                || actual_argument == &Type::Unknown
                                || self.storage_types_compatible(expected_argument, actual_argument)
                        },
                    )
            }
            (
                Type::GenericInstance {
                    name,
                    type_arguments,
                },
                Type::Custom(constructor),
            )
            | (
                Type::Custom(constructor),
                Type::GenericInstance {
                    name,
                    type_arguments,
                },
            ) => self.instantiation_matches(constructor, name, type_arguments),
            (Type::Custom(first_value), Type::Custom(second_value)) => first_value == second_value,
            (Type::Array(expected_inner), Type::Array(actual_inner)) => {
                self.types_compatible(expected_inner, actual_inner)
            }
            (
                Type::FixedSizedArray(expected_inner, expected_length),
                Type::FixedSizedArray(actual_inner, actual_length),
            ) => {
                expected_length == actual_length
                    && self.types_compatible(expected_inner, actual_inner)
            }
            (Type::FixedSizedArray(expected_inner, _), Type::Array(actual_inner))
            | (Type::Array(expected_inner), Type::FixedSizedArray(actual_inner, _)) => {
                self.types_compatible(expected_inner, actual_inner)
            }
            (
                Type::Simd {
                    element: expected_element,
                    lanes: expected_lanes,
                },
                Type::Simd {
                    element: actual_element,
                    lanes: actual_lanes,
                },
            ) => {
                expected_lanes == actual_lanes
                    && self.types_compatible(expected_element, actual_element)
            }
            (
                Type::Reference {
                    lifetime: expected_lifetime,
                    is_mut: expected_mut,
                    is_unique: expected_unique,
                    inner: expected_inner,
                },
                Type::Reference {
                    lifetime: actual_lifetime,
                    is_mut: actual_mut,
                    is_unique: actual_unique,
                    inner: actual_inner,
                },
            ) => {
                (!expected_mut || *actual_mut)
                    && (!expected_unique || *actual_unique)
                    && lifetimes_compatible(expected_lifetime, actual_lifetime)
                    && self.storage_types_compatible(expected_inner, actual_inner)
            }
            (
                Type::RawPointer {
                    is_mut: expected_mut,
                    inner: expected_inner,
                },
                Type::Reference {
                    is_mut: actual_mut,
                    inner: actual_inner,
                    ..
                },
            ) => {
                (!expected_mut || *actual_mut)
                    && self.storage_types_compatible(expected_inner, actual_inner)
            }
            (Type::I64, Type::I32) | (Type::USize, Type::I32) => true,
            _ => false,
        }
    }

    pub fn is_integer_primitive(&self, value_type: &Type) -> bool {
        let resolved = self.resolve(value_type);
        crate::is_integer_type!(resolved)
    }

    // Borrowed storage requires an unchanged element representation.
    pub(crate) fn storage_types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        let expected = self.resolve(expected);
        let actual = self.resolve(actual);

        if expected == actual {
            return true;
        }

        match (&expected, &actual) {
            (Type::Unknown | Type::Infer(_) | Type::TypeParameter(_), _)
            | (_, Type::Unknown | Type::Infer(_) | Type::TypeParameter(_)) => true,
            (Type::Array(left), Type::Array(right)) => self.storage_types_compatible(left, right),
            (
                Type::FixedSizedArray(left, left_length),
                Type::FixedSizedArray(right, right_length),
            ) => left_length == right_length && self.storage_types_compatible(left, right),
            (Type::Tuple(left), Type::Tuple(right)) => {
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right)
                        .all(|(left, right)| self.storage_types_compatible(left, right))
            }
            (
                Type::Simd {
                    element: left,
                    lanes: left_lanes,
                },
                Type::Simd {
                    element: right,
                    lanes: right_lanes,
                },
            ) => left_lanes == right_lanes && self.storage_types_compatible(left, right),
            (
                Type::GenericInstance {
                    type_arguments: left,
                    ..
                },
                Type::GenericInstance {
                    type_arguments: right,
                    ..
                },
            ) => {
                self.types_compatible(&expected, &actual)
                    && left.len() == right.len()
                    && left
                        .iter()
                        .zip(right)
                        .all(|(left, right)| self.storage_types_compatible(left, right))
            }
            (Type::Reference { .. }, Type::Reference { .. }) => {
                self.types_compatible(&expected, &actual)
            }
            (
                Type::RawPointer {
                    is_mut: left_mut,
                    inner: left,
                },
                Type::RawPointer {
                    is_mut: right_mut,
                    inner: right,
                },
            ) => left_mut == right_mut && self.storage_types_compatible(left, right),
            (Type::Custom(_), Type::Custom(_)) => self.types_compatible(&expected, &actual),
            _ => false,
        }
    }

    pub fn unwrap_type_references(mut value_type: Type) -> Type {
        loop {
            match value_type {
                Type::Reference { inner, .. } | Type::RawPointer { inner, .. } => {
                    value_type = *inner
                }
                _ => return value_type,
            }
        }
    }

    pub(super) fn instantiation_matches(
        &self,
        concrete: &str,
        generic: &str,
        arguments: &[Type],
    ) -> bool {
        self.symbols.records.instantiations().iter().any(
            |((registered_generic, registered_arguments), registered_concrete)| {
                registered_concrete == concrete
                    && registered_generic == generic
                    && (registered_arguments.len() == arguments.len()
                        && registered_arguments
                            .iter()
                            .zip(arguments)
                            .all(|(registered, requested)| registered == requested))
            },
        )
    }
}
