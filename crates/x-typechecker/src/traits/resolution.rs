use super::*;

pub(super) struct TraitHierarchy<'a> {
    statements: &'a [Statement],
    visiting: Vec<String>,
}

impl<'a> TraitHierarchy<'a> {
    pub(super) fn new(statements: &'a [Statement]) -> Self {
        Self {
            statements,
            visiting: Vec::new(),
        }
    }
}
