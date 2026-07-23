mod access;

mod model;

use std::collections::{BTreeMap, HashMap};

use super::ContractObligationError;
use super::Function;
use crate::MirTerminator;
use model::{Origin, Value};
use x_ast::StructDefinition;
