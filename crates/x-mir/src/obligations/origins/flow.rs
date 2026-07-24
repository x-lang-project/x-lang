use std::collections::HashMap;

use super::{
    Analysis, access,
    model::{State, entry},
    values,
};
use crate::{BlockIdentifier, MirStatement, MirTerminator};

pub(super) fn transfer_statements(
    analysis: &Analysis<'_, '_>,
    statements: &[MirStatement],
    mut state: State,
) -> State {
    for statement in statements {
        if let MirStatement::Assign(destination, value) = statement {
            if rvalue_call(value).is_some_and(|call| super::havoc::may_mutate(analysis, call)) {
                super::havoc::invalidate(&mut state);
            }

            let result = values::rvalue(analysis, value, &state);
            access::write(analysis, destination, &result, &mut state);
        }
    }

    state
}

pub(super) fn states(analysis: &Analysis<'_, '_>) -> HashMap<BlockIdentifier, State> {
    let body = analysis.body;
    let records = analysis.records;
    let mut entries = HashMap::from([(BlockIdentifier(0), entry(body, records))]);
    let mut pending = vec![BlockIdentifier(0)];

    while let Some(block) = pending.pop() {
        let Some(contents) = body.basic_blocks.get(&block) else {
            continue;
        };

        let Some(state) = entries.get(&block).cloned() else {
            continue;
        };

        let mut state = transfer_statements(analysis, &contents.statements, state);

        if contents
            .terminator
            .as_ref()
            .and_then(terminator_call)
            .is_some_and(|call| super::havoc::may_mutate(analysis, call))
        {
            super::havoc::invalidate(&mut state);
        }

        let targets = match &contents.terminator {
            Some(MirTerminator::Goto(target)) => vec![*target],
            Some(MirTerminator::SwitchInteger {
                targets, otherwise, ..
            }) => targets
                .iter()
                .map(|(_, target)| *target)
                .chain([*otherwise])
                .collect(),
            Some(
                MirTerminator::Call {
                    function,
                    arguments: current_arguments,
                    destination,
                    target,
                }
                | MirTerminator::TryCall {
                    function,
                    arguments: current_arguments,
                    destination,
                    target,
                },
            ) => {
                let result = values::call_value(
                    analysis,
                    crate::CallSite {
                        name: function,
                        arguments: current_arguments,
                        destination: None,
                    },
                    &state,
                );
                access::write(analysis, destination, &result, &mut state);

                vec![*target]
            }
            Some(MirTerminator::CatchCall {
                function,
                arguments: current_arguments,
                fallback,
                destination,
                target,
            }) => {
                let mut result = values::call_value(
                    analysis,
                    crate::CallSite {
                        name: function,
                        arguments: current_arguments,
                        destination: None,
                    },
                    &state,
                );
                result.join(&values::operand(body, fallback, &state, records));
                access::write(analysis, destination, &result, &mut state);

                vec![*target]
            }
            _ => Vec::new(),
        };

        for target in targets {
            let changed = if let Some(existing) = entries.get_mut(&target) {
                let before = existing.clone();

                for (local, value) in &state {
                    existing.entry(*local).or_default().join(value);
                }

                *existing != before
            } else {
                entries.insert(target, state.clone());

                true
            };

            if changed {
                pending.push(target);
            }
        }
    }

    entries
}

fn rvalue_call(value: &crate::Rvalue) -> Option<crate::CallSite<'_>> {
    match value {
        crate::Rvalue::Call {
            function,
            arguments: current_arguments,
        }
        | crate::Rvalue::TryCall {
            function,
            arguments: current_arguments,
        }
        | crate::Rvalue::CatchCall {
            function,
            arguments: current_arguments,
            ..
        } => Some(crate::CallSite {
            name: function,
            arguments: current_arguments,
            destination: None,
        }),
        _ => None,
    }
}

fn terminator_call(value: &MirTerminator) -> Option<crate::CallSite<'_>> {
    match value {
        MirTerminator::Call {
            function,
            arguments: current_arguments,
            ..
        }
        | MirTerminator::TryCall {
            function,
            arguments: current_arguments,
            ..
        }
        | MirTerminator::CatchCall {
            function,
            arguments: current_arguments,
            ..
        } => Some(crate::CallSite {
            name: function,
            arguments: current_arguments,
            destination: None,
        }),
        _ => None,
    }
}
