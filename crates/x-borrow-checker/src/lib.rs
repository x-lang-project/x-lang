#![cfg_attr(
    not(test),
    deny(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unreachable,
        clippy::unwrap_used
    )
)]

use std::sync::Arc;

use context::{AnalysisContext, FunctionLifetimeContext};
use control_flow::state::FlowState;
use program::nominal_owner_name;
use scope::Scope;
use type_properties::{declared_reference_lifetime, has_non_lifetime_generic_parameter};

mod assignments;
mod context;
mod control_flow;
mod errors;
mod expressions;
mod ownership;
mod parameter_origins;
mod place_types;
mod places;
mod program;
mod reference_calls;
mod reference_returns;
mod reference_value;
mod return_summaries;
mod scope;
mod statements;
mod storage;
mod type_properties;
pub use errors::BorrowCheckError;
pub use reference_value::{ReferenceProjection, ReferenceValue};
pub use storage::VariableInfo;

// Origin collection and lifetime checks must share this depth limit.
const MAX_AGGREGATE_LAYOUT_DEPTH: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Region(pub usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Borrow {
    pub region: Region,
    pub is_mut: bool,
    pub is_reborrow: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReferenceTarget {
    pub binding_generation: usize,
    pub projection: Vec<ReferenceProjection>,
    pub target_variable: String,
    pub origin_region: Region,
    pub is_mut: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LoopExitKind {
    Break,
    Continue,
}

#[derive(Clone, PartialEq, Eq)]
struct LoopExit {
    kind: LoopExitKind,
    scopes: Vec<Scope>,
}

#[derive(Clone)]
pub struct BorrowChecker {
    flow: FlowState,
    context: Arc<AnalysisContext>,
}
