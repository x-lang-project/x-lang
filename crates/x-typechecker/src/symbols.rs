mod mangling;
mod records;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use records::RecordCatalogue;
use x_ast::{TraitDefinition as AstTraitDef, Type};
