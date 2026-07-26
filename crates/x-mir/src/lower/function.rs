use std::collections::HashSet;
use std::mem;

use super::MirLowerer;
use crate::errors::MirLowerError;
use crate::{BlockIdentifier, FunctionMeta, GlobalItem, MirBody, MirTerminator};
use x_ast::{Parameter, Statement, Type};
use x_diagnostics::mir_messages;

pub struct FunctionInput {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Vec<Statement>,
    pub meta: FunctionMeta,
}

impl MirLowerer {
    fn block_is_reachable(&self, entry: BlockIdentifier, target: BlockIdentifier) -> bool {
        let mut pending = vec![entry];
        let mut visited = HashSet::new();

        while let Some(block) = pending.pop() {
            if block == target {
                return true;
            }

            if !visited.insert(block) {
                continue;
            }

            match self
                .basic_blocks
                .get(&block)
                .and_then(|contents| contents.terminator.as_ref())
            {
                Some(MirTerminator::Goto(next))
                | Some(MirTerminator::Call { target: next, .. })
                | Some(MirTerminator::TryCall { target: next, .. })
                | Some(MirTerminator::CatchCall { target: next, .. }) => pending.push(*next),
                Some(MirTerminator::SwitchInteger {
                    targets, otherwise, ..
                }) => {
                    pending.extend(targets.iter().map(|(_, next)| *next));
                    pending.push(*otherwise);
                }
                _ => {}
            }
        }

        false
    }
}
