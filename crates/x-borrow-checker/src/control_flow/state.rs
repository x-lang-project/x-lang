use std::sync::Arc;

use crate::{BorrowChecker, LoopExit, Region, scope::Scope};

#[derive(Clone)]
pub(crate) struct FlowState {
    pub(crate) scopes: Vec<Scope>,
    pub(crate) next_region_identifier: usize,
    pub(crate) path_terminated: bool,
    pub(crate) loop_exits: Vec<LoopExit>,
}
