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
