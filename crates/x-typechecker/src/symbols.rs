mod definitions;

mod mangling;
mod records;

pub use definitions::{
    FunctionDefinition, FunctionSignature, MultiFunctionCandidate, TraitInfo, TypeAliasDefinition,
};

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use records::RecordCatalogue;
use x_ast::{TraitDefinition as AstTraitDef, Type};
