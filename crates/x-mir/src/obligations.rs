mod origins;

use std::collections::{BTreeMap, HashMap};

use crate::{BlockIdentifier, GlobalItem, MirBody, MirProgram, MirStatement, Place};
use x_ast::{StructDefinition, Type};

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

pub(super) struct Function<'a> {
    name: &'a str,
    body: Option<&'a MirBody>,
    parameters: &'a [Type],
    pure: bool,
    memoised: bool,
}
