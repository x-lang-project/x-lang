use std::collections::HashSet;

use crate::TypeChecker;
use x_ast::{Parameter, Type};

pub(super) struct FunctionContext {
    pub(super) name: String,
    pub(super) is_pure: bool,
    pub(super) scope_depth: usize,
    pub(super) immutable_parameters: HashSet<String>,
}

#[derive(Clone, Copy, Default)]
pub(super) struct EffectContext {
    in_anonymous_function: bool,
    handles_call: bool,
}

impl TypeChecker {
    pub(super) fn in_anonymous_function(&self) -> bool {
        self.effects.in_anonymous_function
    }

    // Anonymous functions inherit the enclosing namespace and effect policy.
}
