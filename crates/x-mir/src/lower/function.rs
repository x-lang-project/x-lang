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
    pub fn lower_function(&mut self, input: FunctionInput) -> Result<GlobalItem, MirLowerError> {
        let FunctionInput {
            name,
            parameters,
            return_type,
            body: body_statements,
            meta,
        } = input;

        self.source_functions.insert(name.clone());
        self.next_block = 0;
        self.locals.reset();

        self.variable_mappings.clear();
        self.basic_blocks.clear();
        self.loop_targets.clear();

        let return_local = self.allocate_local(Some("_0".to_string()), return_type.clone());
        let argument_count = parameters.len();
        let parameter_types = parameters
            .iter()
            .map(|parameter| parameter.value_type.clone())
            .collect();

        for parameter in parameters {
            let pid = self.allocate_local(Some(parameter.name.clone()), parameter.value_type);
            self.variable_mappings.insert(parameter.name, pid);
        }

        let entry_block = self.make_block();
        self.current_block = Some(entry_block);

        for statement in body_statements {
            self.lower_statement(statement)?;
        }

        if let Some(continuation) = self.current_block {
            if !self.block_is_reachable(entry_block, continuation) {
                self.terminate_block(MirTerminator::Unreachable)?;
            } else if return_type == Type::Void {
                self.terminate_block(MirTerminator::Return)?;
            } else {
                return Err(MirLowerError::UnsupportedExpression(
                    mir_messages::missing_function_return(&name),
                ));
            }
        }

        let body = MirBody {
            call_targets: Default::default(),
            locals: self.locals.take_declarations(),
            basic_blocks: mem::take(&mut self.basic_blocks),
            argument_count,
            return_local,
            is_throws: meta.is_throws,
            is_memoised: meta.is_memoised,
            call_contract: meta.call_contract.clone(),
        };

        self.variable_mappings.clear();

        Ok(GlobalItem::Function {
            name,
            parameter_types,
            return_type,
            body: Some(body),
            meta,
        })
    }

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
