use std::mem;

use super::super::{BorrowCheckError, BorrowChecker, LoopExitKind};
use super::state::FlowState;
use x_ast::{Expression, ExpressionKind, Statement, Type};

pub(crate) enum LoopHeader<'a> {
    Condition(&'a Expression),
    Binding { name: &'a str, value_type: Type },
}

impl BorrowChecker {
    pub(crate) fn check_loop_body(
        &mut self,
        body: &[Statement],
        header: LoopHeader<'_>,
    ) -> Result<(), BorrowCheckError> {
        let enclosing_exits = mem::take(&mut self.flow.loop_exits);
        let result = self.check_loop_iterations(body, header);
        self.flow.loop_exits = enclosing_exits;

        result
    }

    fn check_loop_iterations(
        &mut self,
        body: &[Statement],
        header: LoopHeader<'_>,
    ) -> Result<(), BorrowCheckError> {
        // Check the condition at the loop head on every back edge.
        let before = self.flow.clone();
        let mut head = before.clone();
        let mut exits = Vec::new();

        loop {
            let iteration = self.at_flow(head.clone());
            let paths = iteration.check_loop_iteration(body, &header, &before)?;
            exits.extend(paths.exits);

            let mut next = before.clone();

            for continuation in paths.back_edges {
                let previous = next.clone();
                next.merge_branches(&previous, &continuation);
            }

            if next.scopes == head.scopes {
                self.flow = before.clone();
                self.flow.path_terminated = true;
                for exit in exits {
                    let previous = self.flow.clone();
                    self.flow.merge_branches(&previous, &exit);
                }

                return Ok(());
            }

            head = next;
        }
    }

    fn check_loop_iteration(
        mut self,
        body: &[Statement],
        header: &LoopHeader<'_>,
        before: &FlowState,
    ) -> Result<LoopPaths, BorrowCheckError> {
        let mut paths = LoopPaths::default();

        if let LoopHeader::Condition(condition) = header {
            self.check_expression(condition)?;
        }

        self.collect_loop_exits(before, &mut paths);
        if self.flow.path_terminated {
            return Ok(paths);
        }

        // Include the entry state for zero iterations.
        if header.can_exit() {
            paths.exits.push(self.flow.clone());
        }

        if header.can_enter() {
            self.with_scope(|this| {
                if let LoopHeader::Binding { name, value_type } = header {
                    this.declare_variable(name, value_type.clone(), true)?;
                }

                this.check_statements(body)
            })?;
            self.collect_loop_exits(before, &mut paths);

            if !self.flow.path_terminated {
                paths.back_edges.push(self.flow);
            }
        }

        Ok(paths)
    }

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
