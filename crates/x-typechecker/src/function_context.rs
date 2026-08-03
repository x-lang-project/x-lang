use std::collections::HashSet;

use x_ast::{Parameter, Type};

pub(super) struct FunctionContext {
    pub(super) name: String,
    pub(super) is_pure: bool,
    pub(super) scope_depth: usize,
    pub(super) immutable_parameters: HashSet<String>,
}
