use std::collections::HashSet;

use crate::TypeChecker;
use x_ast::{Parameter, Type};

const MAX_TYPE_RESOLUTION_DEPTH: usize = 64;

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

impl FunctionContext {
    pub(super) fn new(
        name: String,
        is_pure: bool,
        scope_depth: usize,
        parameters: &[Parameter],
    ) -> Self {
        Self {
            name,
            is_pure,
            scope_depth,
            immutable_parameters: parameters
                .iter()
                .filter(|parameter| !parameter.is_mut)
                .map(|parameter| parameter.name.clone())
                .collect(),
        }
    }
}

impl TypeChecker {
    pub(super) fn in_anonymous_function(&self) -> bool {
        self.effects.in_anonymous_function
    }

    pub(crate) fn current_function_name(&self) -> Option<&String> {
        self.function_contexts.last().map(|context| &context.name)
    }

    // Anonymous functions inherit the enclosing namespace and effect policy.
}
