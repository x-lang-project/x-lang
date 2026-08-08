use super::*;

pub(super) fn pattern_names(pattern: &Pattern) -> &[String] {
    match pattern {
        Pattern::Identifier(name) => slice::from_ref(name),
        Pattern::Tuple(names)
        | Pattern::Enum {
            bindings: names, ..
        } => names,
        Pattern::Wildcard => &[],
    }
}
