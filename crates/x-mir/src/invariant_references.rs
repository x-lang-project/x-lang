mod contexts;

use std::collections::HashMap;

use crate::GlobalItem;
use crate::LocalIdentifier;
use crate::MirBody;
use crate::Place;
use x_ast::StructInvariant;

type Contexts = HashMap<LocalIdentifier, Place>;
type Invariants = HashMap<String, StructInvariant>;
