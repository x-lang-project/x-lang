use std::collections::{HashMap, HashSet};

use super::Contexts;
use super::Invariants;
use crate::errors::MirLowerError;
use crate::{LocalIdentifier, MirBody, MirStatement, Operand, Place, Rvalue};
use x_ast::Type;

struct Builder<'a> {
    body: &'a mut MirBody,
    invariants: &'a Invariants,
    definitions: HashMap<LocalIdentifier, Vec<Rvalue>>,
    contexts: Contexts,
    visiting: HashSet<LocalIdentifier>,
    completed: HashSet<LocalIdentifier>,
    prefixes: HashMap<LocalIdentifier, Vec<MirStatement>>,
}
