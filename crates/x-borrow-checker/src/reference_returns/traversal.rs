use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum OriginRequirement {
    DeclaredLifetime,
    Known,
    External,
    TailTransfer,
}

#[derive(Clone, Copy)]
pub(super) struct LifetimeTraversal {
    depth: usize,
    requirement: OriginRequirement,
}

impl LifetimeTraversal {
    pub(super) fn new(requirement: OriginRequirement) -> Self {
        Self {
            depth: 0,
            requirement,
        }
    }

    pub(super) fn descend(self) -> Self {
        Self {
            depth: self.depth + 1,
            ..self
        }
    }
}
