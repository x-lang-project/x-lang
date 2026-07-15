use std::sync::Arc;

use crate::{BorrowChecker, LoopExit, Region, scope::Scope};

#[derive(Clone)]
pub(crate) struct FlowState {
    pub(crate) scopes: Vec<Scope>,
    pub(crate) next_region_identifier: usize,
    pub(crate) path_terminated: bool,
    pub(crate) loop_exits: Vec<LoopExit>,
}

impl Default for FlowState {
    fn default() -> Self {
        Self {
            scopes: vec![Scope {
                region: Region(0),
                ..Default::default()
            }],
            next_region_identifier: 1,
            path_terminated: false,
            loop_exits: Vec::new(),
        }
    }
}

impl FlowState {
    pub(crate) fn merge_branches(&mut self, left: &Self, right: &Self) {
        let mut exits = left.loop_exits.clone();

        for exit in &right.loop_exits {
            if !exits.contains(exit) {
                exits.push(exit.clone());
            }
        }

        self.merge_live_paths(left, right);
        self.loop_exits = exits;
    }

    fn merge_live_paths(&mut self, left: &Self, right: &Self) {
        match (left.path_terminated, right.path_terminated) {
            (true, true) => {
                self.path_terminated = true;
                return;
            }
            (true, false) => {
                *self = right.clone();
                return;
            }
            (false, true) => {
                *self = left.clone();
                return;
            }
            (false, false) => {}
        }

        for (index, scope) in self.scopes.iter_mut().enumerate() {
            let Some(left_scope) = left.scopes.get(index) else {
                continue;
            };

            let Some(right_scope) = right.scopes.get(index) else {
                continue;
            };

            scope.merge_branches(left_scope, right_scope);
        }
    }
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

    pub(crate) fn merge_branch_states(&mut self, left: &Self, right: &Self) {
        self.flow.merge_branches(&left.flow, &right.flow);
    }
}
