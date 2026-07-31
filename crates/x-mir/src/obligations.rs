mod effects;
mod origins;
mod owners;

use std::collections::{BTreeMap, HashMap};

use crate::{BlockIdentifier, GlobalItem, MirBody, MirProgram, MirStatement, Place};
use x_ast::{StructDefinition, Type};

pub use effects::LoweredMemoryEffects;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ContractObligationError {
    #[error("invalid invariant owner context: {0}")]
    OwnerContext(String),
    #[error(
        "function `{function}` block {block} tail argument {argument} has frame-local or unresolved reference origin"
    )]
    TailStorage {
        function: String,
        block: usize,
        argument: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutationObligation {
    pub block: BlockIdentifier,
    pub statement: usize,
    pub destination: Place,
    /// Invariant owners, innermost first.
    pub owners: Vec<Place>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TailTransferObligation {
    pub block: BlockIdentifier,
    pub reference_arguments: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FunctionObligations {
    pub memory: LoweredMemoryEffects,
    pub source_pure: bool,
    pub tail_transfers: Vec<TailTransferObligation>,
    pub mutations: Vec<MutationObligation>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContractObligations {
    pub functions: BTreeMap<String, FunctionObligations>,
}

pub(super) struct Function<'a> {
    name: &'a str,
    body: Option<&'a MirBody>,
    parameters: &'a [Type],
    pure: bool,
    memoised: bool,
}

fn collect<'a>(items: &'a [GlobalItem], functions: &mut Vec<Function<'a>>) {
    for item in items {
        match item {
            GlobalItem::Function {
                name,
                body,
                parameter_types,
                meta,
                ..
            } => functions.push(Function {
                name,
                body: body.as_ref(),
                parameters: parameter_types,
                pure: meta.is_pure,
                memoised: meta.is_memoised,
            }),
            GlobalItem::Impl { methods, .. } => collect(methods, functions),
            _ => {}
        }
    }
}

impl MirProgram {
    /// Recompute after every MIR transformation.
    pub fn contract_obligations(&self) -> Result<ContractObligations, ContractObligationError> {
        self.validate_invariant_reference_contexts()
            .map_err(|error| ContractObligationError::OwnerContext(error.to_string()))?;
        let mut functions = Vec::new();
        collect(&self.items, &mut functions);
        let records: HashMap<&str, &StructDefinition> = self
            .items
            .iter()
            .filter_map(|item| {
                if let GlobalItem::Struct(definition) = item {
                    Some((definition.name.as_str(), definition))
                } else {
                    None
                }
            })
            .collect();
        let mutating_calls = origins::mutating_calls(&functions, &records);
        let summaries = origins::return_summaries(&functions, &records);
        let unproved = origins::unproved_memory(&functions, &records, &summaries, &mutating_calls);
        let effects = effects::transitive_effects(&functions, &unproved);
        let mut result = ContractObligations::default();

        for function in functions {
            let mut obligations = FunctionObligations {
                source_pure: function.pure,
                memory: effects.get(function.name).copied().unwrap_or_default(),
                ..FunctionObligations::default()
            };

            if let Some(body) = function.body {
                obligations.tail_transfers =
                    origins::validate_transfers(&function, &records, &summaries, &mutating_calls)?;

                for (block, contents) in &body.basic_blocks {
                    for (statement, instruction) in contents.statements.iter().enumerate() {
                        if let MirStatement::Assign(destination, _) = instruction {
                            let owners = destination
                                .invariant_owner_chain()
                                .into_iter()
                                .filter(|owner| {
                                    let value_type = body.place_type(owner);
                                    let value_type = value_type.pointee().unwrap_or(&value_type);
                                    let name = match value_type {
                                        Type::Custom(name) | Type::GenericInstance { name, .. } => {
                                            Some(name.as_str())
                                        }
                                        _ => None,
                                    };

                                    name.and_then(|name| records.get(name))
                                        .is_some_and(|record| record.invariant.is_some())
                                })
                                .cloned()
                                .collect();
                            obligations.mutations.push(MutationObligation {
                                block: *block,
                                statement,
                                destination: destination.clone(),
                                owners,
                            });
                        }
                    }
                }
            }

            result.functions.insert(function.name.into(), obligations);
        }

        Ok(result)
    }
}
