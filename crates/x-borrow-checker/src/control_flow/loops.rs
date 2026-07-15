use std::mem;

use super::super::{BorrowCheckError, BorrowChecker, LoopExitKind};
use super::state::FlowState;
use x_ast::{Expression, ExpressionKind, Statement, Type};

pub(crate) enum LoopHeader<'a> {
    Condition(&'a Expression),
    Binding { name: &'a str, value_type: Type },
}

impl BorrowChecker {
    fn collect_loop_exits(&mut self, before: &FlowState, paths: &mut LoopPaths) {
        for exit in mem::take(&mut self.flow.loop_exits) {
            let mut state = before.clone();
            state.scopes = exit.scopes;
            state.path_terminated = false;

            match exit.kind {
                LoopExitKind::Break => paths.exits.push(state),
                LoopExitKind::Continue => paths.back_edges.push(state),
            }
        }
    }
}

#[derive(Default)]
struct LoopPaths {
    exits: Vec<FlowState>,
    back_edges: Vec<FlowState>,
}

impl LoopHeader<'_> {
    fn can_enter(&self) -> bool {
        !matches!(
            self,
            Self::Condition(Expression {
                kind: ExpressionKind::Boolean(false),
                ..
            })
        )
    }

    fn can_exit(&self) -> bool {
        !matches!(
            self,
            Self::Condition(Expression {
                kind: ExpressionKind::Boolean(true),
                ..
            })
        )
    }
}
