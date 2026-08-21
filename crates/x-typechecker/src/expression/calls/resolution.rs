use super::*;

impl TypeChecker {
    pub(in crate::expression) fn multi_candidate_matches(
        &self,
        expected: &Type,
        actual: &Type,
    ) -> bool {
        let expected = self.resolve(expected);
        let actual = self.resolve(actual);

        match (&expected, &actual) {
            (first_value, second_value) if first_value == second_value => true,
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            (Type::TypeParameter(_), _) | (_, Type::TypeParameter(_)) => true,
            (Type::Custom(first_value), Type::Custom(second_value)) => first_value == second_value,
            (
                Type::Reference {
                    is_mut: expected_mut,
                    is_unique: expected_unique,
                    inner: expected_inner,
                    ..
                },
                Type::Reference {
                    is_mut: actual_mut,
                    is_unique: actual_unique,
                    inner: actual_inner,
                    ..
                },
            ) => {
                expected_mut == actual_mut
                    && expected_unique == actual_unique
                    && self.multi_candidate_matches(expected_inner, actual_inner)
            }
            (
                Type::RawPointer {
                    is_mut: expected_mut,
                    inner: expected_inner,
                },
                Type::RawPointer {
                    is_mut: actual_mut,
                    inner: actual_inner,
                },
            ) => {
                (!expected_mut || *actual_mut)
                    && self.multi_candidate_matches(expected_inner, actual_inner)
            }
            (Type::Array(expected_inner), Type::Array(actual_inner)) => {
                self.multi_candidate_matches(expected_inner, actual_inner)
            }
            (
                Type::FixedSizedArray(expected_inner, expected_length),
                Type::FixedSizedArray(actual_inner, actual_length),
            ) => {
                expected_length == actual_length
                    && self.multi_candidate_matches(expected_inner, actual_inner)
            }
            (Type::Tuple(expected_items), Type::Tuple(actual_items)) => {
                expected_items.len() == actual_items.len()
                    && expected_items
                        .iter()
                        .zip(actual_items)
                        .all(|(expected, actual)| self.multi_candidate_matches(expected, actual))
            }
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
                expected_name == actual_name
                    && expected_arguments.len() == actual_arguments.len()
                    && expected_arguments
                        .iter()
                        .zip(actual_arguments)
                        .all(|(expected, actual)| self.multi_candidate_matches(expected, actual))
            }
            _ => false,
        }
    }
}
