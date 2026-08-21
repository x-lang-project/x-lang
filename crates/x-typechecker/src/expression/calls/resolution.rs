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
