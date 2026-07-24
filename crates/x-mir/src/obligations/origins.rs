use std::collections;

mod access;
mod flow;
mod havoc;
mod memory;
mod model;
mod values;

use std::collections::{BTreeMap, HashMap};

use super::{ContractObligationError, Function, TailTransferObligation};
use crate::MirTerminator;
use model::{Origin, Value};
use x_ast::StructDefinition;

type Records<'a> = HashMap<&'a str, &'a StructDefinition>;
type Summaries = BTreeMap<String, Value>;

struct Analysis<'a, 'r> {
    body: &'a crate::MirBody,
    records: &'a Records<'r>,
    summaries: &'a Summaries,
    mutating_calls: &'a collections::BTreeSet<String>,
}
pub(super) use memory::unproved_memory;
pub(super) fn mutating_calls(
    functions: &[Function<'_>],
    records: &Records<'_>,
) -> collections::BTreeSet<String> {
    havoc::mutating_calls(functions, records)
}

pub(super) fn return_summaries(functions: &[Function<'_>], records: &Records<'_>) -> Summaries {
    let mutating_calls = havoc::mutating_calls(functions, records);
    let mut summaries: Summaries = functions
        .iter()
        .filter(|function| function.body.is_some())
        .map(|function| (function.name.into(), Value::default()))
        .collect();

    loop {
        let previous = summaries.clone();

        for function in functions {
            let Some(body) = function.body else { continue };

            let analysis = Analysis {
                body,
                records,
                summaries: &previous,
                mutating_calls: &mutating_calls,
            };
            let states = flow::states(&analysis);
            let mut result = Value::default();

            for (block, state) in states {
                let Some(contents) = body.basic_blocks.get(&block) else {
                    continue;
                };

                let state = flow::transfer_statements(&analysis, &contents.statements, state);

                match &contents.terminator {
                    Some(MirTerminator::Return) => {
                        result.join(&state.get(&body.return_local).cloned().unwrap_or_default())
                    }
                    Some(MirTerminator::TailCall {
                        function: callee,
                        arguments: current_arguments,
                    }) => {
                        result.join(&values::call_value(
                            &analysis,
                            crate::CallSite {
                                name: callee,
                                arguments: current_arguments,
                                destination: None,
                            },
                            &state,
                        ));
                    }
                    _ => {}
                }
            }

            let result = body
                .locals
                .get(&body.return_local)
                .map(|local| access::canonical_value(&local.value_type, &result, records))
                .unwrap_or_default();
            summaries.insert(function.name.into(), result);
        }

        if summaries == previous {
            return summaries;
        }
    }
}
