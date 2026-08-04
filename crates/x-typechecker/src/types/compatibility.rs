use super::*;

impl TypeChecker {
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
