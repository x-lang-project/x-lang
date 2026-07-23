use std::collections;

mod access;

mod model;

use std::collections::{BTreeMap, HashMap};

use super::ContractObligationError;
use super::Function;
use crate::MirTerminator;
use model::{Origin, Value};
use x_ast::StructDefinition;

type Records<'a> = HashMap<&'a str, &'a StructDefinition>;
type Summaries = BTreeMap<String, Value>;

struct Analysis<'a, 'r> {
    body: &'a crate::MirBody,
    records: &'a Records<'r>,
    summaries: &'a Summaries,
    mutating_calls: &'a collections::BTreeSet<String>,
}
