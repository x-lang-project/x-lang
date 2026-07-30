mod contexts;

mod rewrite;

use std::collections::HashMap;

use crate::errors::MirLowerError;
use crate::{GlobalItem, LocalIdentifier, MirBody, MirProgram, Place};
use x_ast::StructInvariant;

type Contexts = HashMap<LocalIdentifier, Place>;
type Invariants = HashMap<String, StructInvariant>;

fn unsupported(detail: &str) -> MirLowerError {
    MirLowerError::InvalidContract(
        x_diagnostics::mir_messages::invariant_reference_context_required(detail),
    )
}
