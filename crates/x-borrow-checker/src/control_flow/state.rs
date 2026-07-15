use std::sync::Arc;

use crate::{BorrowChecker, LoopExit, Region, scope::Scope};

#[derive(Clone)]
pub(crate) struct FlowState {
    pub(crate) scopes: Vec<Scope>,
    pub(crate) next_region_identifier: usize,
    pub(crate) path_terminated: bool,
    pub(crate) loop_exits: Vec<LoopExit>,
}

impl BorrowChecker {
    pub(crate) fn at_flow(&self, flow: FlowState) -> Self {
        Self {
            flow,
            context: Arc::clone(&self.context),
        }
    }

    pub(crate) fn fork(&self) -> Self {
        self.at_flow(self.flow.clone())
    }
}
