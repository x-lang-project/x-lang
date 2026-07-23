use std::collections::{BTreeSet, HashMap};

use super::{
    Analysis, Records,
    model::{Origin, State, type_paths},
};
use crate::obligations::Function;
use crate::{CallSite, MirBody, MirStatement, MirTerminator, Place};
use x_ast::Type;

pub(super) fn may_mutate(analysis: &Analysis<'_, '_>, call: CallSite<'_>) -> bool {
    // Opaque callees can mutate escaped pointers through globals.
    !analysis.summaries.contains_key(call.name) || analysis.mutating_calls.contains(call.name)
}
